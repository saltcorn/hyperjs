pub mod body_response;
pub mod builder;
pub mod status;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{body::Body, version::Version};
use body_response::BodyResponse;
use builder::ResponseBuilder;
use status::StatusCode;

#[napi]
#[derive(Debug, Clone)]
pub struct Response {
  inner: BodyResponse,
}

impl From<BodyResponse> for Response {
  fn from(value: BodyResponse) -> Self {
    Self { inner: value }
  }
}

#[napi]
impl Response {
  #[napi]
  pub fn builder() -> ResponseBuilder {
    ResponseBuilder::default()
  }

  #[napi(constructor)]
  pub fn new(body: &Body) -> Self {
    let request: BodyResponse = body.into();
    Self::from(request)
  }

  #[napi(factory)]
  pub fn from_parts() {
    unimplemented!()
  }

  #[napi]
  pub fn status(&mut self) -> StatusCode {
    self.inner.status()
  }

  #[napi]
  pub fn version(&self) -> Version {
    self.inner.version()
  }

  #[napi]
  pub fn headers(&self, env: Env) -> Result<Object<'_>> {
    self.inner.headers(&env)
  }

  #[napi]
  pub fn body(&self) -> Body {
    self.inner.body()
  }
}
