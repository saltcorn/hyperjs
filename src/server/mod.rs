pub mod complete_request;
mod get_next_id;
mod handle_http_request;
mod request_context;

use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::tokio::{TokioIo, TokioTimer};
use napi::threadsafe_function::ThreadsafeFunction;
use napi::{bindgen_prelude::*, Error, Result, Status};
use napi_derive::napi;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot;

use crate::{response::Response, router::Router};
use handle_http_request::handle_http_request;
use request_context::RequestContext;

// Global state for pending requests
lazy_static::lazy_static! {
  static ref PENDING_REQUESTS: Arc<std::sync::Mutex<std::collections::HashMap<u32, oneshot::Sender<Response>>>> =
    Arc::new(std::sync::Mutex::new(std::collections::HashMap::new()));
  static ref NEXT_ID: Arc<std::sync::Mutex<u32>> = Arc::new(std::sync::Mutex::new(0));
}

type AppThreadsafeFunction =
  Arc<ThreadsafeFunction<RequestContext, (), RequestContext, Status, false, false, 0>>;

/// HTTP Server that integrates with JavaScript handlers via Router
#[napi]
pub struct Server {
  router: Router,
  handler_fn: Option<AppThreadsafeFunction>,
}

#[napi]
impl Server {
  /// Create a new server with a router
  #[napi(constructor)]
  pub fn new(router: &Router) -> Result<Self> {
    Ok(Self {
      router: router.clone(),
      handler_fn: None,
    })
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
    let handler_fn = self
      .handler_fn
      .clone()
      .ok_or_else(|| Error::new(Status::GenericFailure, "Handler not set"))?;

    std::thread::spawn(move || {
      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async move {
        let listener = TcpListener::bind(&addr).await.unwrap();
        println!("Server listening on {}", addr);

        loop {
          let (socket, _) = listener.accept().await.unwrap();
          let io = TokioIo::new(socket);
          let handler_fn = handler_fn.clone();

          tokio::task::spawn(async move {
            let _ = http1::Builder::new()
              .timer(TokioTimer::new())
              .serve_connection(
                io,
                service_fn(move |req| handle_http_request(req, handler_fn.clone())),
              )
              .await;
          });
        }
      });
    });

    Ok(())
  }

  #[napi]
  pub fn get_router(&self) -> Router {
    self.router.clone()
  }
}
