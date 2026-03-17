mod get_next_id;
mod handle_http_request;

use env_logger::Builder as EnvLoggerBuilder;
use hyper::Method as LibMethod;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::tokio::{TokioIo, TokioTimer};
use log::LevelFilter;
use matchit::{InsertError, Router};
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction};
use napi_derive::napi;
use std::sync::Arc;
use tokio::net::TcpListener;

use crate::request::Request;
use crate::response::Response;
use handle_http_request::handle_http_request;

// Global state for pending requests
lazy_static::lazy_static! {
  static ref NEXT_ID: Arc<std::sync::Mutex<u32>> = Arc::new(std::sync::Mutex::new(0));
}

type JsHandlerFn<'a> =
  Function<'a, FnArgs<(Request, Response)>, Either<Either<bool, ()>, Promise<Either<bool, ()>>>>;

type ThreadsafeMiddlewareFn = ThreadsafeFunction<
  FnArgs<(Request, Response)>,
  Either<Either<bool, ()>, Promise<Either<bool, ()>>>,
  FnArgs<(Request, Response)>,
  Status,
  false,
  false,
  0,
>;

#[derive(Clone)]
pub struct MiddlewareMeta {
  /// The string used to register the middleware in the router.
  ///
  /// None: Indicates globally registered middleware
  ///
  /// If Some, associated function (`handler`) is only executed if value
  /// returned from router matches this value
  route: Option<String>,

  /// Function use to handle middleware
  ///
  /// Returns:
  ///   true => run the next middleware
  ///   _ => don't run the next middleware
  handler: Arc<ThreadsafeMiddlewareFn>,

  /// The HTTP method to match from the request.
  ///
  /// If Some, associated function (`handler`) is only executed if Request's
  /// method matches this value
  method: Option<LibMethod>,
}

/// HTTP Server that integrates with JavaScript handlers via Router
#[napi]
pub struct Server {
  middlewares: Vec<MiddlewareMeta>,
  router: Router<String>,
}

impl Server {
  fn register_middleware(
    &mut self,
    route: Option<String>,
    handler: JsHandlerFn,
    _env: Env,
  ) -> Result<()> {
    let tsfn = handler
      .build_threadsafe_function()
      .build_callback(|ctx: ThreadsafeCallContext<FnArgs<(Request, Response)>>| Ok(ctx.value))?;
    self.middlewares.push(MiddlewareMeta {
      route,
      handler: Arc::new(tsfn),
      method: None,
    });
    Ok(())
  }

  fn register_route(
    &mut self,
    route: String,
    handler: JsHandlerFn,
    method: LibMethod,
  ) -> Result<()> {
    let tsfn = handler
      .build_threadsafe_function()
      .build_callback(|ctx: ThreadsafeCallContext<FnArgs<(Request, Response)>>| Ok(ctx.value))?;
    if let Err(e) = self.router.insert(route.to_owned(), route.to_owned()) {
      match e {
        InsertError::Conflict { .. } => {}
        _ => return Err(Error::new(Status::GenericFailure, e.to_string())),
      }
    }
    self.middlewares.push(MiddlewareMeta {
      route: Some(route),
      handler: Arc::new(tsfn),
      method: Some(method),
    });
    Ok(())
  }
}

#[napi]
impl Server {
  /// Create a new server with a router
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    Ok(Self {
      middlewares: Vec::new(),
      router: Router::new(),
    })
  }

  #[napi]
  pub fn delete(&mut self, route: String, handler: JsHandlerFn) -> Result<()> {
    self.register_route(route, handler, LibMethod::DELETE)
  }

  #[napi]
  pub fn get(&mut self, route: String, handler: JsHandlerFn) -> Result<()> {
    self.register_route(route, handler, LibMethod::GET)
  }

  #[napi]
  pub fn post(&mut self, route: String, handler: JsHandlerFn) -> Result<()> {
    self.register_route(route, handler, LibMethod::POST)
  }

  #[napi]
  pub fn put(&mut self, route: String, handler: JsHandlerFn) -> Result<()> {
    self.register_route(route, handler, LibMethod::PUT)
  }

  #[napi(js_name = "use")]
  pub fn uze(&mut self, route: Option<String>, middleware: JsHandlerFn, env: Env) -> Result<()> {
    self.register_middleware(route, middleware, env)
  }

  #[napi]
  pub fn listen(&self, addr: String) -> Result<()> {
    let router = Arc::new(self.router.clone());
    let middlewares = Arc::new(self.middlewares.clone());

    EnvLoggerBuilder::new()
      .filter_level(LevelFilter::max())
      .init();

    std::thread::spawn(move || {
      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async move {
        let listener = TcpListener::bind(&addr).await.unwrap();
        log::debug!("Server listening on {}", addr);

        loop {
          let (socket, _) = listener.accept().await.unwrap();
          let io = TokioIo::new(socket);

          let router = router.clone();
          let middlewares = middlewares.clone();

          tokio::task::spawn(async move {
            let _ = http1::Builder::new()
              .timer(TokioTimer::new())
              .serve_connection(
                io,
                service_fn(move |req| {
                  handle_http_request(req, router.clone(), middlewares.clone())
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
