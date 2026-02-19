mod append;
mod attachment;
mod clear_cookie;
mod content_type;
mod cookie;
mod cookie_options;
mod download;
mod formut;
mod get;
mod json;
mod links;
mod location;
mod redirect;
mod send;
mod send_file;
mod send_status;
mod set;
mod status;
pub mod status_code;
mod vary;
mod wrapped_response;

use std::sync::{Arc, Mutex};

use bytes::Bytes;
use napi::bindgen_prelude::*;
use napi_derive::napi;

pub use wrapped_response::{CrateBody, WrappedResponse};

use crate::request::Request;

#[napi]
#[derive(Clone, Default)]
pub struct Response {
  inner: Arc<Mutex<WrappedResponse>>,
  request: Request,
}

impl Response {
  pub fn new(request: Request, inner: Option<WrappedResponse>) -> Self {
    Self {
      request,
      inner: Arc::new(Mutex::new(inner.unwrap_or_default())),
    }
  }

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
    Self::default()
  }

  #[napi]
  pub fn end(&mut self, data: Option<Either3<String, Buffer, Uint8Array>>) -> Result<()> {
    let data = data.map(|data| match data {
      Either3::A(data) => Bytes::copy_from_slice(data.as_bytes()),
      Either3::B(data) => Bytes::from_owner(data),
      Either3::C(data) => Bytes::from_owner(data),
    });
    self.with_inner(|response| response.end(data))
  }

  #[napi(getter)]
  pub fn req(&self) -> Request {
    self.request.to_owned()
  }

  // TODO: jsonp()
  // TODO: render()
}
