mod append;
mod attachment;
mod clear_cookie;
mod content_type;
mod cookie;
mod cookie_options;
mod get;
mod json;
mod send;
mod send_status;
mod status;
pub mod status_code;
mod wrapped_response;

use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi_derive::napi;

pub use wrapped_response::WrappedResponse;

#[napi]
#[derive(Clone, Debug)]
pub struct Response {
  inner: Arc<Mutex<WrappedResponse>>,
}

impl From<WrappedResponse> for Response {
  fn from(value: WrappedResponse) -> Self {
    Self {
      inner: Arc::new(Mutex::new(value)),
    }
  }
}

impl Response {
  pub fn with_inner<F, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(&mut WrappedResponse) -> Result<T>,
  {
    match self.inner.lock() {
      Ok(mut inner) => f(&mut inner),
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("Could not obtain lock on response. {e}"),
      )),
    }
  }
}

#[napi]
impl Response {
  #[napi(constructor)]
  pub fn get_test_instance() -> Self {
    WrappedResponse::default().into()
  }

  // TODO: download()
  // TODO: end()
  // TODO: format()
  // TODO: jsonp()
  // TODO: links()
  // TODO: location()
  // TODO: redirect()
  // TODO: render()
  // TODO: sendFile()
  // TODO: set()
  // TODO: vary()
}
