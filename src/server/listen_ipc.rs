#[cfg(unix)]
use std::path::Path;
use std::sync::Arc;
#[cfg(unix)]
use std::{fs::Permissions, os::unix::fs::PermissionsExt};

use env_logger::Builder as EnvLoggerBuilder;
use hyper_util::rt::TokioIo;
use log::LevelFilter;
use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunctionCallMode},
};
use napi_derive::napi;
#[cfg(unix)]
use tokio::net::UnixListener;
#[cfg(windows)]
use tokio::net::windows::named_pipe::ServerOptions;

#[cfg(unix)]
use super::systemd_notify::systemd_notify;
use super::{Server, create_handler_task::create_handler_task};
use crate::server::ThreadsafeCallbackFn;

#[napi(object)]
pub struct IpcServerListenOptions {
  /// The maximum number of pending connections that can be queued before the OS
  /// starts refusing new ones. Passed directly to the underlying `listen(2)` syscall.
  ///
  /// A value of `0` lets the OS choose a reasonable default.
  ///
  /// Default = 0
  pub backlog: u32,

  /// Unix domain socket path the server should listen on.
  ///
  /// Ignored if [`port`] is also specified — TCP takes precedence.
  ///
  /// Defaults to `None`.
  ///
  /// [`port`]: Self::port
  pub path: String,

  ///  Makes the pipe readable for all users. Default: false.
  pub readable_all: Option<bool>,

  /// Makes the pipe writable for all users. Default: false.
  pub writable_all: Option<bool>,
}

#[cfg(unix)]
struct SetSocketPermissionsParams<P: AsRef<Path>> {
  path: P,
  readable_all: bool,
  writeable_all: bool,
}

#[cfg(unix)]
fn set_socket_permissions<P: AsRef<Path>>(params: SetSocketPermissionsParams<P>) -> Result<()> {
  let permissions_mode = match (params.readable_all, params.writeable_all) {
    // owner read+write only
    (false, false) => 0o600,

    // all users read+write — necessary for non-owner clients to connect
    (true, true) => 0o666,

    // read without write is insufficient for clients to connect to a Unix socket
    (true, false) => {
      return Err(Error::from_reason(
        "Invalid combination (readable_all = true, writeable_all = false): 
    connecting to a Unix socket requires both read and write permissions, 
    so granting read alone to all users does not enable them to connect.",
      ));
    }

    // write without read is insufficient for clients to connect to a Unix socket
    (false, true) => {
      return Err(Error::from_reason(
        "Invalid combination (readable_all = false, writeable_all = true): 
    connecting to a Unix socket requires both read and write permissions, 
    so granting write alone to all users does not enable them to connect.",
      ));
    }
  };
  let perm = Permissions::from_mode(permissions_mode);
  std::fs::set_permissions(params.path, perm)?;
  Ok(())
}

fn setup_logging() {
  EnvLoggerBuilder::new()
    .filter_level(LevelFilter::max())
    .init();
}

#[napi]
impl Server {
  #[cfg(unix)]
  pub fn _listen_ipc_unix(
    &self,
    options: IpcServerListenOptions,
    callback: ThreadsafeCallbackFn,
  ) -> Result<()> {
    let ipc_listener = UnixListener::bind(&options.path)
      .map_err(|e| Error::from_reason(format!("Error creating IPC listener. {e}")))?;

    set_socket_permissions(SetSocketPermissionsParams {
      path: options.path.to_owned(),
      readable_all: options.readable_all.unwrap_or_default(),
      writeable_all: options.writable_all.unwrap_or_default(),
    })?;

    log::info!("IPC server listening on {}", options.path);

    let router = Arc::new(self.router.clone());
    let middlewares = Arc::new(self.middlewares.clone());

    setup_logging();

    std::thread::spawn(move || {
      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async move {
        let addr = ipc_listener.local_addr().unwrap();
        let mut addr_str = addr.as_pathname().and_then(|p| p.to_str());
        if cfg!(any(target_os = "linux", target_os = "android")) {
          addr_str = addr_str.or(
            addr
              .as_abstract_name()
              .and_then(|bytes| str::from_utf8(bytes).ok()),
          )
        }
        let addr = addr_str.unwrap_or_default();
        let server_status_message = format!("Server listening on '{}'", addr);
        log::debug!("{server_status_message}");

        systemd_notify(&server_status_message);

        callback.call(addr.to_string(), ThreadsafeFunctionCallMode::Blocking);

        loop {
          let (stream, _) = ipc_listener.accept().await.unwrap();
          let io = TokioIo::new(stream);
          let router = router.clone();
          let middlewares = middlewares.clone();

          create_handler_task(Box::new(io), router, middlewares);
        }
      });
    });

    Ok(())
  }

  #[cfg(windows)]
  pub fn _listen_ipc_windows(
    &self,
    options: IpcServerListenOptions,
    callback: ThreadsafeCallbackFn,
  ) -> Result<()> {
    let router = Arc::new(self.router.clone());
    let middlewares = Arc::new(self.middlewares.clone());
    let pipe_path = options.path.clone();

    setup_logging();

    let mut server = ServerOptions::new()
      .first_pipe_instance(true)
      .create(options.path)?;

    std::thread::spawn(move || {
      let rt = tokio::runtime::Runtime::new().unwrap();
      rt.block_on(async move {
        callback.call(pipe_path.to_owned(), ThreadsafeFunctionCallMode::Blocking);

        loop {
          server.connect().await.unwrap();
          let connected_client = server;

          // Construct the next server to be connected before sending the one
          // we already have of onto a task. This ensures that the server
          // isn't closed (after it's done in the task) before a new one is
          // available. Otherwise the client might error with
          // `io::ErrorKind::NotFound`.
          server = ServerOptions::new().create(pipe_path.clone()).unwrap();
          let io = TokioIo::new(connected_client);
          let router = router.clone();
          let middlewares = middlewares.clone();

          create_handler_task(io, router, middlewares);
        }
      });
    });

    Ok(())
  }

  #[napi]
  pub fn listen_ipc(
    &self,
    options: IpcServerListenOptions,
    callback: Function<String, ()>,
  ) -> Result<()> {
    let ts_callback = callback.build_threadsafe_function().build_callback(
      |ctx: ThreadsafeCallContext<String>| {
        #[allow(clippy::unit_arg)]
        Ok(ctx.value)
      },
    )?;

    #[cfg(unix)]
    self._listen_ipc_unix(options, ts_callback)?;

    #[cfg(windows)]
    self._listen_ipc_windows(options, ts_callback)?;

    Ok(())
  }
}
