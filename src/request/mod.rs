mod body_request;
mod builder;
mod method;

use hyper::Request as LibRequest;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{body::Body, version::Version};
use body_request::BodyRequest;
use builder::Builder;
use method::Method;

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

  #[napi(factory)]
  pub fn get(uri: String) -> Builder {
    Builder::from(LibRequest::get::<String>(uri))
  }

  #[napi(factory)]
  pub fn put(uri: String) -> Builder {
    Builder::from(LibRequest::put::<String>(uri))
  }

  #[napi(factory)]
  pub fn post(uri: String) -> Builder {
    Builder::from(LibRequest::post::<String>(uri))
  }

  #[napi(factory)]
  pub fn delete(uri: String) -> Builder {
    Builder::from(LibRequest::delete::<String>(uri))
  }

  #[napi(factory)]
  pub fn options(uri: String) -> Builder {
    Builder::from(LibRequest::options::<String>(uri))
  }

  #[napi(factory)]
  pub fn head(uri: String) -> Builder {
    Builder::from(LibRequest::head::<String>(uri))
  }

  #[napi(factory)]
  pub fn connect(uri: String) -> Builder {
    Builder::from(LibRequest::connect::<String>(uri))
  }

  #[napi(factory)]
  pub fn patch(uri: String) -> Builder {
    Builder::from(LibRequest::patch::<String>(uri))
  }

  #[napi(factory)]
  pub fn trace(uri: String) -> Builder {
    Builder::from(LibRequest::trace::<String>(uri))
  }

  #[napi(constructor)]
  pub fn new(body: &Body) -> Self {
    let request: BodyRequest = body.into();
    Self::from(request)
  }

  #[napi(factory)]
  pub fn from_parts() {
    unimplemented!()
  }

  #[napi]
  pub fn method(&mut self) -> Method {
    self.inner.method()
  }

  pub fn uri(&self) -> String {
    self.inner.uri()
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
