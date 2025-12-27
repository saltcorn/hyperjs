mod get_next_id;
mod handle_http_request;
mod request_context;

use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::tokio::{TokioIo, TokioTimer};
use matchit::Router;
use napi::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction};
use napi::{bindgen_prelude::*, Error, Result, Status};
use napi_derive::napi;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;

use crate::request::Request;
use crate::response::Response;
use handle_http_request::handle_http_request;
use request_context::RequestContext;

// Global state for pending requests
lazy_static::lazy_static! {
  static ref NEXT_ID: Arc<std::sync::Mutex<u32>> = Arc::new(std::sync::Mutex::new(0));
}

type AppThreadsafeFunction =
  Arc<ThreadsafeFunction<RequestContext, (), RequestContext, Status, false, false, 0>>;

type ThreadsafeRequestHandlerFn = Arc<
  ThreadsafeFunction<Request, Promise<&'static mut Response>, Request, Status, false, false, 0>,
>;

/// HTTP Server that integrates with JavaScript handlers via Router
#[napi]
pub struct Server {
  get_router: Arc<RwLock<Router<ThreadsafeRequestHandlerFn>>>,
  handler_fn: Option<AppThreadsafeFunction>,
}

#[napi]
impl Server {
  /// Create a new server with a router
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    Ok(Self {
      get_router: Arc::new(RwLock::new(Router::new())),
      handler_fn: None,
    })
  }

  #[napi]
  pub fn get(
    &mut self,
    route: String,
    handler: Function<Request, Promise<&'static mut Response>>,
  ) -> Result<()> {
    let tsfn = handler
      .build_threadsafe_function()
      .build_callback(|ctx: ThreadsafeCallContext<Request>| Ok(ctx.value))?;

    self
      .get_router
      .write()
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?
      .insert(route, Arc::new(tsfn))
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(())
  }

  #[napi]
  pub fn set_handler(&mut self, handler: Function<RequestContext, ()>) -> Result<()> {
    println!("Setting JS handler function");

    let tsfn = handler
      .build_threadsafe_function()
      .build_callback(|ctx| Ok(ctx.value))?;

    self.handler_fn = Some(Arc::new(tsfn));
    Ok(())
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
