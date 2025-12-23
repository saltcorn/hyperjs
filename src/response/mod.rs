mod body_response;
mod builder;
mod status;

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{body::Body, version::Version};
use body_response::BodyResponse;
use builder::Builder;
use status::StatusCode;

#[napi]
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
  #[napi(factory)]
  pub fn builder() -> Builder {
    Builder::default()
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

  pub fn version(&self) -> Version {
    self.inner.version()
  }

  pub fn headers(&self, env: Env) -> Result<Object<'_>> {
    self.inner.headers(&env)
  }

  pub fn body(&self) -> Body {
    self.inner.body()
  }
}
