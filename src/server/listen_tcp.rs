use std::{
  future,
  net::{IpAddr, SocketAddr},
};

use napi::{bindgen_prelude::*, threadsafe_function::ThreadsafeCallContext};
use napi_derive::napi;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::net::TcpListener;

use super::Server;

#[napi(object)]
pub struct TcpServerListenOptions {
  /// The maximum number of pending connections that can be queued before the OS
  /// starts refusing new ones. Passed directly to the underlying `listen(2)` syscall.
  ///
  /// A value of `0` lets the OS choose a reasonable default.
  ///
  /// Default = 0
  pub backlog: Option<u32>,

  /// The local address the server will bind to.
  ///
  /// Defaults to `"0.0.0.0"` (all IPv4 interfaces) when [`ipv6_only`] is `false`,
  /// or `"::"` (all IPv6 interfaces) when [`ipv6_only`] is `true`.
  ///
  /// [`ipv6_only`]: Self::ipv6_only
  pub host: Option<String>,

  /// When `true`, opens an IPv6-only socket (`AF_INET6` with `IPV6_V6ONLY` enabled),
  /// rejecting any IPv4 connections. When `false` (the default), a dual-stack socket
  /// is used that accepts both IPv4 and IPv6 connections.
  ///
  /// Defaults to `false`.
  pub ipv6_only: Option<bool>,

  /// When `true`, sets `SO_REUSEPORT` on the socket, allowing multiple sockets to
  /// bind to the same address and port. Useful for load-balancing incoming connections
  /// across several threads or processes.
  ///
  /// Not supported on all platforms (e.g. older Windows versions).
  ///
  /// Defaults to `false`.
  pub reuse_port: Option<bool>,

  /// The TCP port the server should bind to. When set, takes precedence over [`path`].
  ///
  /// A value of `0` asks the OS to assign an available ephemeral port, which can then
  /// be retrieved after binding.
  ///
  /// Defaults to `0`.
  ///
  /// [`path`]: Self::path
  pub port: Option<u16>,
}

pub struct LibTcpServerListenOptions {
  backlog: i32,
  host: String,
  ipv6_only: bool,
  #[cfg(all(unix, windows))]
  reuse_port: bool,
  port: u16,
}

impl From<TcpServerListenOptions> for LibTcpServerListenOptions {
  fn from(options: TcpServerListenOptions) -> Self {
    Self {
      backlog: options.backlog.unwrap_or(0) as i32,
      host: match options.ipv6_only.unwrap_or_default() {
        true => "::".to_owned(),
        false => "0.0.0.0".to_owned(),
      },
      ipv6_only: options.ipv6_only.unwrap_or_default(),
      #[cfg(all(unix, windows))]
      reuse_port: options.ipv6_only.unwrap_or_default(),
      port: options.port.unwrap_or(0),
    }
  }
}

fn host_to_ip_addr(host: &str) -> Result<IpAddr> {
  host
    .parse()
    .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid host: {e}")))
}

struct CreateTcpSocketParams {
  #[cfg(all(unix, windows))]
  reuse_port: bool,
  ipv6_only: bool,
}

fn create_tcp_socket(params: CreateTcpSocketParams) -> Result<Socket> {
  let domain = match params.ipv6_only {
    true => Domain::IPV6,
    false => Domain::IPV4,
  };
  let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP))
    .map_err(|e| Error::from_reason(e.to_string()))?;
  socket
    .set_only_v6(params.ipv6_only)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  #[cfg(all(unix, windows))]
  socket
    .set_reuse_port(params.reuse_port)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  Ok(socket)
}

struct CreateTcpListenerParams {
  addr: SocketAddr,
  backlog: i32,
  socket: CreateTcpSocketParams,
}

fn create_tcp_listener(params: CreateTcpListenerParams) -> Result<TcpListener> {
  let socket = create_tcp_socket(params.socket)?;
  socket
    .bind(&params.addr.into())
    .map_err(|e| Error::from_reason(e.to_string()))?;
  socket
    .listen(params.backlog)
    .map_err(|e| Error::from_reason(e.to_string()))?;
  TcpListener::from_std(socket.into()).map_err(|e| Error::from_reason(e.to_string()))
}

#[napi]
impl Server {
  #[napi]
  pub fn listen_tcp(
    &self,
    options: TcpServerListenOptions,
    callback: Function<String, ()>,
  ) -> Result<()> {
    let options: LibTcpServerListenOptions = options.into();

    let create_tcp_listener_params = CreateTcpListenerParams {
      addr: SocketAddr::from((host_to_ip_addr(&options.host)?, options.port)),
      backlog: options.backlog,
      socket: CreateTcpSocketParams {
        ipv6_only: options.ipv6_only,
        #[cfg(all(unix, windows))]
        reuse_port: options.reuse_port,
      },
    };
    let tcp_listener = create_tcp_listener(create_tcp_listener_params)?;
    let create_tcp_listener = move || future::ready(Ok(tcp_listener));
    let ts_callback = callback.build_threadsafe_function().build_callback(
      |ctx: ThreadsafeCallContext<String>| {
        #[allow(clippy::unit_arg)]
        Ok(ctx.value)
      },
    )?;
    self._listen_tcp(create_tcp_listener, ts_callback);
    Ok(())
  }
}
