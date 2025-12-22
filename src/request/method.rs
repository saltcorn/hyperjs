use hyper::http::method::Method as LibMethod;
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct Method {
  inner: LibMethod,
}

impl From<&Method> for LibMethod {
  fn from(value: &Method) -> Self {
    value.inner.clone()
  }
}

impl From<LibMethod> for Method {
  fn from(value: LibMethod) -> Self {
    Self { inner: value }
  }
}

#[napi]
impl Method {
  #[napi(factory)]
  pub fn connect() -> Self {
    Self::from(LibMethod::CONNECT)
  }

  #[napi(factory)]
  pub fn delete() -> Self {
    Self::from(LibMethod::DELETE)
  }

  #[napi(factory)]
  pub fn get() -> Self {
    Self::from(LibMethod::GET)
  }

  #[napi(factory)]
  pub fn head() -> Self {
    Self::from(LibMethod::HEAD)
  }

  #[napi(factory)]
  pub fn options() -> Self {
    Self::from(LibMethod::OPTIONS)
  }

  #[napi(factory)]
  pub fn patch() -> Self {
    Self::from(LibMethod::PATCH)
  }

  #[napi(factory)]
  pub fn post() -> Self {
    Self::from(LibMethod::POST)
  }

  #[napi(factory)]
  pub fn put() -> Self {
    Self::from(LibMethod::PUT)
  }

  #[napi(factory)]
  pub fn trace() -> Self {
    Self::from(LibMethod::TRACE)
  }

  #[napi(factory)]
  pub fn from_bytes(src: Uint8Array) -> Result<Self> {
    LibMethod::from_bytes(src.as_ref())
      .map(Self::from)
      .map_err(|e| Error::new(Status::InvalidArg, e.to_string()))
  }

  #[napi]
  pub fn is_indempotent(&mut self) -> bool {
    self.inner.is_idempotent()
  }

  #[napi]
  pub fn as_js_string(&mut self) -> String {
    self.inner.to_string()
  }
}
