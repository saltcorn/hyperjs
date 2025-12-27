mod get_next_id;
mod handle_http_request;
mod request_context;

use hyper::Method as LibMethod;
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::tokio::{TokioIo, TokioTimer};
use matchit::Router;
use napi::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction};
use napi::{bindgen_prelude::*, Error, Result, Status};
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
  ThreadsafeFunction<Request, Promise<&'static mut Response>, Request, Status, false, false, 0>,
>;

type RoutersMap = Arc<RwLock<HashMap<LibMethod, Router<ThreadsafeRequestHandlerFn>>>>;

/// HTTP Server that integrates with JavaScript handlers via Router
#[napi]
pub struct Server {
  get_router: RoutersMap,
}

impl Server {
  fn register_route(
    &mut self,
    route: String,
    handler: Function<Request, Promise<&'static mut Response>>,
    method: LibMethod,
  ) -> Result<()> {
    let tsfn = handler
      .build_threadsafe_function()
      .build_callback(|ctx: ThreadsafeCallContext<Request>| Ok(ctx.value))?;
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
  pub fn get(
    &mut self,
    route: String,
    handler: Function<Request, Promise<&'static mut Response>>,
  ) -> Result<()> {
    self.register_route(route, handler, LibMethod::GET)
  }

  #[napi]
  pub fn post(
    &mut self,
    route: String,
    handler: Function<Request, Promise<&'static mut Response>>,
  ) -> Result<()> {
    self.register_route(route, handler, LibMethod::POST)
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
