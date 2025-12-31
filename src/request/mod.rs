pub mod body;
pub mod body_ref;
pub mod builder;
pub mod interface;
pub mod method;

use std::collections::HashMap;

use hyper::Request as LibRequest;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::version::Version;
use body::RequestBody;
use body_ref::RequestBodyRef;
use builder::RequestBuilder;
use interface::RequestInterface;
use method::Method;

#[napi]
#[derive(Debug)]
pub struct Request {
  inner: Box<dyn RequestInterface>,
  params: HashMap<String, String>,
}

impl From<Box<dyn RequestInterface>> for Request {
  fn from(value: Box<dyn RequestInterface>) -> Self {
    Self {
      inner: value,
      params: HashMap::with_capacity(0),
    }
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
  pub fn new(body: &mut RequestBody) -> Result<Self> {
    RequestBuilder::new().body(body)
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
  pub fn body(&self, request: Reference<Request>, env: Env) -> Result<RequestBodyRef> {
    let shared_body_ref = request.share_with(env, |request| Ok(request.inner.body()))?;
    Ok(RequestBodyRef::new(shared_body_ref))
  }

  #[napi(getter)]
  pub fn params(&self, env: Env) -> Result<Object<'_>> {
    let mut headers_obj = Object::new(&env)?;
    for (key, value) in &self.params {
      headers_obj.set(key, value)?;
    }
    Ok(headers_obj)
  }
}

impl Request {
  pub fn set_param(&mut self, k: String, v: String) {
    self.params.insert(k, v);
  }

  pub fn set_params<I, K, V>(&mut self, iterator: I)
  where
    I: Iterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
  {
    for (k, v) in iterator {
      self.params.insert(k.into(), v.into());
    }
  }
}
