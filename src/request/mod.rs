pub mod method;
mod params;
mod wrapped_request;

use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi_derive::napi;

pub use wrapped_request::WrappedRequest;

#[napi]
#[derive(Clone)]
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
  pub fn with_inner<F, T>(&self, f: F) -> Result<T>
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
}

#[napi]
impl Request {
  #[napi(getter)]
  pub fn body(&self) -> Result<Option<String>> {
    self.with_inner(|req| Ok(req.body.to_owned()))
  }
}
