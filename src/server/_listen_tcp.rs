use env_logger::Builder as EnvLoggerBuilder;
use futures::prelude::*;

use hyper::rt::{Read, Write};
use hyper::{server::conn::http1, service::service_fn};
use hyper_util::rt::tokio::{TokioIo, TokioTimer};
use log::LevelFilter;
use matchit::Router;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use rustls_acme::AcmeConfig;
use rustls_acme::caches::DirCache;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

use super::{MiddlewareMeta, Server, ThreadsafeCallbackFn, handle_http_request};

impl Server {
  pub(super) fn _listen_tcp<F, Fut>(&self, create_tcp_listener: F, callback: ThreadsafeCallbackFn)
  where
    F: FnOnce() -> Fut + Send + 'static,
    Fut: Future<Output = Result<TcpListener>> + Send,
  {
    let router = Arc::new(self.router.clone());
    let middlewares = Arc::new(self.middlewares.clone());
    let acme_config_meta = self.acme_config_meta.clone();

    EnvLoggerBuilder::new()
      .filter_level(LevelFilter::max())
      .init();

    std::thread::spawn(move || {
      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async move {
        let tcp_listener = create_tcp_listener().await.unwrap();
        let addr = tcp_listener.local_addr().unwrap();
        let server_status_message = format!("Server listening on {}", addr);
        log::debug!("{server_status_message}");

        #[cfg(unix)]
        systemd_notify(&server_status_message);

        callback.call(addr.to_string(), ThreadsafeFunctionCallMode::Blocking);

        match acme_config_meta {
          Some(acme) => {
            let tcp_stream = TcpListenerStream::new(tcp_listener);

            let mut tls_incoming = AcmeConfig::new(acme.domains)
              .contact_push(format!("mailto:{}", acme.contact_email))
              .cache(DirCache::new(acme.cache_dir))
              .tokio_incoming(tcp_stream, Vec::new());

            while let Some(tls) = tls_incoming.next().await {
              let tls = match tls {
                Ok(t) => t,
                Err(e) => {
                  log::error!("TLS accept error: {}", e);
                  continue;
                }
              };

              let io = TokioIo::new(tls);
              let router = router.clone();
              let middlewares = middlewares.clone();

              create_handler_task(io, router, middlewares);
            }
          }
          None => loop {
            let (socket, _) = tcp_listener.accept().await.unwrap();
            let io = TokioIo::new(socket);
            let router = router.clone();
            let middlewares = middlewares.clone();

            create_handler_task(Box::new(io), router, middlewares);
          },
        }
      });
    });
  }
}

#[cfg(unix)]
fn systemd_notify(server_status_message: &str) {
  use sd_notify::{NotifyState, notify};
  if let Err(e) = notify(&[NotifyState::Ready]) {
    log::error!("Failed to notify systemd: {}", e);
  }

  let _ = notify(&[NotifyState::Status(server_status_message)]);
}

fn create_handler_task<I: Read + Write + Unpin + Send + 'static>(
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
