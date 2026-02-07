mod accepts;
pub mod error;
mod get;
mod method;
mod params;
mod range;
mod wrapped_request;

use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi_derive::napi;

pub use wrapped_request::WrappedRequest;

use crate::utilities;

#[napi]
#[derive(Clone, Debug, Default)]
pub struct Request {
  inner: Arc<Mutex<WrappedRequest>>,
}

impl From<WrappedRequest> for Request {
  fn from(value: WrappedRequest) -> Self {
    Self {
      inner: Arc::new(Mutex::new(value)),
    }
  }
}

impl Request {
  pub fn with_inner_mut<F, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(&mut WrappedRequest) -> Result<T>,
  {
    match self.inner.lock() {
      Ok(mut inner) => f(&mut inner),
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("Could not obtain lock on request. {e}"),
      )),
    }
  }

  pub fn with_inner<F, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(&WrappedRequest) -> Result<T>,
  {
    match self.inner.lock() {
      Ok(inner) => f(&inner),
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("Could not obtain lock on request. {e}"),
      )),
    }
  }
}

#[napi]
impl Request {
  /// Included for test purposes. Normally, you will obtain a request from the
  /// server
  #[napi(constructor)]
  pub fn get_test_instance() -> Self {
    Self::default()
  }

  /// `req.body`'s shape is based on user-controlled input, all properties and
  /// values in this object are untrusted and should be validated before
  /// trusting. For example, `req.body.foo.toString()` may fail in multiple
  /// ways, for example the foo property may not be there or may not be a
  /// string, and `toString` may not be a function and instead a string or
  /// other user input.
  #[napi(getter)]
  pub fn body(&self, env: Env) -> Result<Either4<String, Unknown<'static>, Buffer, ()>> {
    let body = self.with_inner_mut(|req| Ok(req.body.to_owned()))?;
    match body {
      None => Ok(Either4::D(())),
      Some(body) => match body {
        Either3::A(body) => Ok(Either4::A(body)),
        Either3::B(json_value) => utilities::json_to_napi(env, json_value).map(Either4::B),
        Either3::C(body) => Ok(Either4::C(Buffer::from(body))),
      },
    }
  }

  //   Properties
  //   TODO: app
  //   TODO: baseUrl
  //   TODO: body
  //   TODO: cookies
  //   TODO: fresh
  //   TODO: host
  //   TODO: hostname
  //   TODO: ip
  //   TODO: ips
  //   TODO: originalUrl
  //   TODO: path
  //   TODO: protocol
  //   TODO: query
  //   TODO: res
  //   TODO: route
  //   TODO: secure
  //   TODO: signedCookies
  //   TODO: stale
  //   TODO: subdomains
  //   TODO: xhr

  // Methods
  //   TODO: acceptsCharsets
  //   TODO: acceptsEncodings
  //   TODO: acceptsLanguages
  //   TODO: is
  //   TODO: range
}
