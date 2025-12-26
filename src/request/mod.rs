pub mod body_request;
pub mod builder;
pub mod method;

use hyper::Request as LibRequest;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{body::Body, version::Version};
use body_request::BodyRequest;
use builder::RequestBuilder;
use method::Method;

#[napi]
#[derive(Debug, Clone)]
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
  pub fn builder() -> RequestBuilder {
    RequestBuilder::new()
  }

  #[napi(factory)]
  pub fn get(uri: String) -> RequestBuilder {
    RequestBuilder::from(LibRequest::get::<String>(uri))
  }

  #[napi(factory)]
  pub fn put(uri: String) -> RequestBuilder {
    RequestBuilder::from(LibRequest::put::<String>(uri))
  }

  #[napi(factory)]
  pub fn post(uri: String) -> RequestBuilder {
    RequestBuilder::from(LibRequest::post::<String>(uri))
  }

  #[napi(factory)]
  pub fn delete(uri: String) -> RequestBuilder {
    RequestBuilder::from(LibRequest::delete::<String>(uri))
  }

  #[napi(factory)]
  pub fn options(uri: String) -> RequestBuilder {
    RequestBuilder::from(LibRequest::options::<String>(uri))
  }

  #[napi(factory)]
  pub fn head(uri: String) -> RequestBuilder {
    RequestBuilder::from(LibRequest::head::<String>(uri))
  }

  #[napi(factory)]
  pub fn connect(uri: String) -> RequestBuilder {
    RequestBuilder::from(LibRequest::connect::<String>(uri))
  }

  #[napi(factory)]
  pub fn patch(uri: String) -> RequestBuilder {
    RequestBuilder::from(LibRequest::patch::<String>(uri))
  }

  #[napi(factory)]
  pub fn trace(uri: String) -> RequestBuilder {
    RequestBuilder::from(LibRequest::trace::<String>(uri))
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

  #[napi]
  pub fn uri(&self) -> String {
    self.inner.uri()
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
