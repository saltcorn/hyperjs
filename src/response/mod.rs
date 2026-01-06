mod append;
mod attachment;
mod clear_cookie;
mod content_type;
mod cookie;
mod cookie_options;
mod get;
mod json;
pub mod response_ref;
mod send;
mod send_status;
mod status;
pub mod status_code;

use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::{Error as LibError, Response as LibResponse};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::utilities::{empty, full};

type ResponseInner = LibResponse<BoxBody<Bytes, LibError>>;

#[napi]
#[derive(Debug)]
pub struct Response {
  inner: Option<ResponseInner>,
}

impl Default for Response {
  fn default() -> Self {
    Self {
      inner: Some(LibResponse::new(empty())),
    }
  }
}

impl From<ResponseInner> for Response {
  fn from(value: ResponseInner) -> Self {
    Self { inner: Some(value) }
  }
}

impl Response {
  pub fn inner(&mut self) -> Result<&mut ResponseInner> {
    self.inner.as_mut().ok_or(Error::new(
      Status::GenericFailure,
      "Misuse of consumed response.",
    ))
  }

  pub fn take(&mut self) -> Result<ResponseInner> {
    self.inner.take().ok_or(Error::new(
      Status::GenericFailure,
      "Misuse of consumed response.",
    ))
  }

  pub fn end(&mut self, data: Bytes) -> Result<()> {
    let response = self.take()?.map(|_| full(data));
    self.inner = Some(response);
    Ok(())
  }
}

#[napi]
impl Response {
  #[napi(constructor)]
  pub fn new() -> Response {
    Self::default()
  }
}
