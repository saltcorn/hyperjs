mod _listen_tcp;
mod create_handler_task;
mod get_next_id;
mod handle_http_request;
mod listen_ipc;
mod listen_tcp;
mod register_middleware;
mod register_route;
mod systemd_notify;

use env_logger::Builder as EnvLoggerBuilder;
use futures::prelude::*;
use hyper::Method as LibMethod;
use hyper_util::rt::TokioIo;
use log::LevelFilter;
use matchit::Router;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeFunction;
use napi_derive::napi;
use rustls_acme::AcmeConfig;
use rustls_acme::caches::DirCache;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;

use crate::request::Request;
use crate::response::Response;
use create_handler_task::create_handler_task;
use handle_http_request::handle_http_request;
#[cfg(unix)]
use systemd_notify::systemd_notify;

// Global state for pending requests
lazy_static::lazy_static! {
  static ref NEXT_ID: Arc<std::sync::Mutex<u32>> = Arc::new(std::sync::Mutex::new(0));
}

pub type JsHandlerFn<'a> =
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

type ThreadsafeCallbackFn = ThreadsafeFunction<String, (), String, Status, false, false, 0>;

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

#[napi(object)]
#[derive(Debug, Clone)]
pub struct AcmeConfigMeta {
  pub domains: Vec<String>,
  pub contact_email: String,
  pub cache_dir: String,
}

/// HTTP Server that integrates with JavaScript handlers via Router
#[napi]
pub struct Server {
  middlewares: Vec<MiddlewareMeta>,
  router: Router<String>,
  acme_config_meta: Option<AcmeConfigMeta>,
}

#[napi]
impl Server {
  /// Create a new server with a router
  #[napi(constructor)]
  pub fn new() -> Result<Self> {
    Ok(Self {
      middlewares: Vec::new(),
      router: Router::new(),
      acme_config_meta: None,
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
  pub fn acme_config_meta(&mut self, config: AcmeConfigMeta) {
    self.acme_config_meta = Some(config)
  }

  #[napi]
  pub fn listen(&self, addr: String) -> Result<()> {
    let router = Arc::new(self.router.clone());
    let middlewares = Arc::new(self.middlewares.clone());
    let acme_config_meta = self.acme_config_meta.clone();

    EnvLoggerBuilder::new()
      .filter_level(LevelFilter::max())
      .init();

    std::thread::spawn(move || {
      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async move {
        let tcp_listener = TcpListener::bind(&addr).await.unwrap();
        let server_status_message = format!("Server listening on {}", addr);
        log::debug!("{server_status_message}");

        #[cfg(unix)]
        systemd_notify(&server_status_message);

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

    Ok(())
  }
}
