mod body;
mod builder;
mod method;
mod version;

use hyper::Request as LibRequest;
use napi_derive::napi;

use builder::Builder;

pub enum BodyRequest {
  Empty(LibRequest<()>),
  String(LibRequest<String>),
}

impl From<LibRequest<()>> for BodyRequest {
  fn from(value: LibRequest<()>) -> Self {
    BodyRequest::Empty(value)
  }
}

impl From<LibRequest<String>> for BodyRequest {
  fn from(value: LibRequest<String>) -> Self {
    BodyRequest::String(value)
  }
}

#[napi]
pub struct Request {
  inner: BodyRequest,
}

impl From<BodyRequest> for Request {
  fn from(value: BodyRequest) -> Self {
    Self { inner: value }
  }
}

#[napi]
impl Request {
  #[napi(factory)]
  pub fn builder() -> Builder {
    Builder::new()
  }
}
