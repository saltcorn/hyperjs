mod get_next_id;
mod handle_http_request;

use hyper::Method as LibMethod;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::tokio::{TokioIo, TokioTimer};
use matchit::Router;
use napi::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction};
use napi::{Error, Result, Status, bindgen_prelude::*};
use napi_derive::napi;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;

use crate::request::Request;
use crate::response::Response;
use handle_http_request::handle_http_request;

// Global state for pending requests
lazy_static::lazy_static! {
  static ref NEXT_ID: Arc<std::sync::Mutex<u32>> = Arc::new(std::sync::Mutex::new(0));
}

type ThreadsafeRequestHandlerFn = Arc<
  ThreadsafeFunction<
    FnArgs<(Request, Response)>,
    Either<(), Promise<()>>,
    FnArgs<(Request, Response)>,
    Status,
    false,
    false,
    0,
  >,
>;

type RoutersMap = Arc<RwLock<HashMap<LibMethod, Router<ThreadsafeRequestHandlerFn>>>>;

type JsHandlerFunction<'a> = Function<'a, FnArgs<(Request, Response)>, Either<(), Promise<()>>>;

/// HTTP Server that integrates with JavaScript handlers via Router
#[napi]
pub struct Server {
  get_router: RoutersMap,
}

impl Server {
  fn register_middleware(
    &mut self,
    route: Option<String>,
    handler: Function<(Request, Response)>,
    method: LibMethod,
  ) -> Result<()> {
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
      .get_router
      .write()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    let get_router = routers_map.get_mut(&method);
    match get_router {
      Some(router) => {
        router
          .insert(route, Arc::new(tsfn))
          .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
      }
      None => {
        let mut router = Router::new();
        router
          .insert(route, Arc::new(tsfn))
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
      get_router: Arc::new(RwLock::new(HashMap::new())),
    })
  }

  #[napi]
  pub fn get(&mut self, route: String, handler: JsHandlerFunction) -> Result<()> {
    self.register_route(route, handler, LibMethod::GET)
  }

  #[napi]
  pub fn post(&mut self, route: String, handler: JsHandlerFunction) -> Result<()> {
    self.register_route(route, handler, LibMethod::POST)
  }

  #[napi(js_name = "use")]
  pub fn uze(
    &mut self,
    route: Option<String>,
    middleware: Function<(Request, Response)>,
  ) -> Result<()> {
    self.register_middleware(route, middleware, LibMethod::POST)
  }

  #[napi]
  pub fn listen(&self, addr: String) -> Result<()> {
    let router = self.get_router.clone();

    std::thread::spawn(move || {
      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async move {
        let listener = TcpListener::bind(&addr).await.unwrap();
        println!("Server listening on {}", addr);

        loop {
          let (socket, _) = listener.accept().await.unwrap();
          let io = TokioIo::new(socket);

          let router = router.clone();

          tokio::task::spawn(async move {
            let _ = http1::Builder::new()
              .timer(TokioTimer::new())
              .serve_connection(
                io,
                service_fn(move |req| handle_http_request(req, router.clone())),
              )
              .await;
          });
        }
      });
    });

    Ok(())
  }
}
