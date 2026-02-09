mod get_next_id;
mod handle_http_request;

use hyper::Method as LibMethod;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::tokio::{TokioIo, TokioTimer};
use matchit::Router;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction};
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, RwLock};
use tokio::net::TcpListener;

use crate::request::Request;
use crate::response::Response;
use handle_http_request::handle_http_request;

// Global state for pending requests
lazy_static::lazy_static! {
  static ref NEXT_ID: Arc<std::sync::Mutex<u32>> = Arc::new(std::sync::Mutex::new(0));
}

type RoutersMap = Arc<RwLock<HashMap<LibMethod, Router<Arc<RouteMeta>>>>>;

type MiddlewaresRouter = Arc<RwLock<Router<MiddlewaresMeta>>>;

type JsHandlerFunction<'a> = Function<'a, FnArgs<(Request, Response)>, Either<(), Promise<()>>>;

type ThreadsafeRequestHandlerFn = ThreadsafeFunction<
  FnArgs<(Request, Response)>,
  Either<(), Promise<()>>,
  FnArgs<(Request, Response)>,
  Status,
  false,
  false,
  0,
>;

type JsMiddlewareFunction<'a> =
  Function<'a, FnArgs<(Request, Response)>, Either<bool, Promise<bool>>>;

type ThreadsafeMiddlewareFn = ThreadsafeFunction<
  FnArgs<(Request, Response)>,
  Either<bool, Promise<bool>>,
  FnArgs<(Request, Response)>,
  Status,
  false,
  false,
  0,
>;

#[derive(Clone)]
pub struct MiddlewaresMeta {
  pub next_called: Arc<Mutex<bool>>,
  middlewares: Vec<Arc<ThreadsafeMiddlewareFn>>,
}

impl MiddlewaresMeta {
  fn new() -> Self {
    Self {
      next_called: Arc::new(Mutex::new(false)),
      middlewares: Vec::with_capacity(0),
    }
  }
}

pub struct RouteMeta {
  handler: ThreadsafeRequestHandlerFn,
}

impl RouteMeta {
  pub fn new(handler: ThreadsafeRequestHandlerFn) -> Self {
    Self { handler }
  }
}

/// HTTP Server that integrates with JavaScript handlers via Router
#[napi]
pub struct Server {
  middlewares_meta: MiddlewaresRouter,
  routers_map: RoutersMap,
  app_wide_middlewares: Vec<Arc<ThreadsafeMiddlewareFn>>,
}

impl Server {
  fn register_middleware(
    &mut self,
    route: Option<String>,
    handler: JsMiddlewareFunction,
    _env: Env,
  ) -> Result<()> {
    // TODO: Support root level middlewares
    let route = match route {
      Some(route) => route,
      None => {
        let tsfn = handler.build_threadsafe_function().build_callback(
          |ctx: ThreadsafeCallContext<FnArgs<(Request, Response)>>| Ok(ctx.value),
        )?;
        self.app_wide_middlewares.push(Arc::new(tsfn));
        return Ok(());
      }
    };
    let mut middlewares_meta = self
      .middlewares_meta
      .write()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    let mut route_middlewares = match middlewares_meta.remove(&route) {
      Some(middleware) => middleware,
      None => MiddlewaresMeta::new(),
    };
    let tsfn = handler
      .build_threadsafe_function()
      .build_callback(|ctx: ThreadsafeCallContext<FnArgs<(Request, Response)>>| Ok(ctx.value))?;

    route_middlewares.middlewares.push(Arc::new(tsfn));

    middlewares_meta
      .insert(route, route_middlewares)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    Ok(())
  }

  fn register_route(
    &mut self,
    route: String,
    handler: JsHandlerFunction,
    method: LibMethod,
  ) -> Result<()> {
    let tsfn = handler
      .build_threadsafe_function()
      .build_callback(|ctx: ThreadsafeCallContext<FnArgs<(Request, Response)>>| Ok(ctx.value))?;
    let mut routers_map = self
      .routers_map
      .write()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    let router = routers_map.get_mut(&method);
    match router {
      Some(router) => {
        router
          .insert(route, Arc::new(RouteMeta::new(tsfn)))
          .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
      }
      None => {
        let mut router = Router::new();
        router
          .insert(route, Arc::new(RouteMeta::new(tsfn)))
          .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        routers_map.insert(method, router);
      }
    }
    Ok(())
  }
}

#[napi]
impl Server {
  /// Create a new server with a router
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    Ok(Self {
      routers_map: Arc::new(RwLock::new(HashMap::new())),
      middlewares_meta: Arc::new(RwLock::new(Router::new())),
      app_wide_middlewares: Vec::new(),
    })
  }

  #[napi]
  pub fn delete(&mut self, route: String, handler: JsHandlerFunction) -> Result<()> {
    self.register_route(route, handler, LibMethod::DELETE)
  }

  #[napi]
  pub fn get(&mut self, route: String, handler: JsHandlerFunction) -> Result<()> {
    self.register_route(route, handler, LibMethod::GET)
  }

  #[napi]
  pub fn post(&mut self, route: String, handler: JsHandlerFunction) -> Result<()> {
    self.register_route(route, handler, LibMethod::POST)
  }

  #[napi]
  pub fn put(&mut self, route: String, handler: JsHandlerFunction) -> Result<()> {
    self.register_route(route, handler, LibMethod::PUT)
  }

  #[napi(js_name = "use")]
  pub fn uze(
    &mut self,
    route: Option<String>,
    middleware: JsMiddlewareFunction,
    env: Env,
  ) -> Result<()> {
    self.register_middleware(route, middleware, env)
  }

  #[napi]
  pub fn listen(&self, addr: String) -> Result<()> {
    let router = self.routers_map.clone();
    let middlewares_router = self.middlewares_meta.clone();
    let app_wide_middlewares = self.app_wide_middlewares.clone();

    std::thread::spawn(move || {
      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async move {
        let listener = TcpListener::bind(&addr).await.unwrap();
        println!("Server listening on {}", addr);

        loop {
          let (socket, _) = listener.accept().await.unwrap();
          let io = TokioIo::new(socket);

          let router = router.clone();
          let middlewares_router = middlewares_router.clone();
          let app_wide_middlewares = app_wide_middlewares.clone();

          tokio::task::spawn(async move {
            let _ = http1::Builder::new()
              .timer(TokioTimer::new())
              .serve_connection(
                io,
                service_fn(move |req| {
                  handle_http_request(
                    req,
                    router.clone(),
                    middlewares_router.clone(),
                    app_wide_middlewares.clone(),
                  )
                }),
              )
              .await;
          });
        }
      });
    });

    Ok(())
  }
}
