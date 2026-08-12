use hyper::rt::{Read, Write};
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::tokio::TokioTimer;
use matchit::Router;
use std::sync::Arc;

use super::{MiddlewareMeta, handle_http_request::handle_http_request};

pub fn create_handler_task<I: Read + Write + Unpin + Send + 'static>(
  io: I,
  router: Arc<Router<String>>,
  middlewares: Arc<Vec<MiddlewareMeta>>,
) {
  tokio::task::spawn(async move {
    let _ = http1::Builder::new()
      .timer(TokioTimer::new())
      .serve_connection(
        io,
        service_fn(move |req| handle_http_request(req, router.clone(), middlewares.clone())),
      )
      .await;
  });
}
