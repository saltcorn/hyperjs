use hyper::http::{method::Method as LibMethod, request::Builder as LibBuilder};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{body::RequestBody, method::Method, Request};
use crate::{request::interface::RequestInterface, version::Version};

#[napi]
pub struct RequestBuilder {
  inner: Option<LibBuilder>,
}

impl From<LibBuilder> for RequestBuilder {
  fn from(value: LibBuilder) -> Self {
    Self { inner: Some(value) }
  }
}

impl RequestBuilder {
  fn take_inner(&mut self) -> Result<LibBuilder> {
    self.inner.take().ok_or(Error::new(
      Status::GenericFailure,
      "Method cannot be called on a consumed builder.",
    ))
  }

  fn get_inner(&mut self) -> Result<&LibBuilder> {
    self.inner.as_ref().ok_or(Error::new(
      Status::GenericFailure,
      "Method cannot be called on a consumed builder.",
    ))
  }
}

impl Default for RequestBuilder {
  fn default() -> Self {
    Self::from(LibBuilder::new())
  }
}

#[napi]
impl RequestBuilder {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self::default()
  }

  #[napi]
  pub fn method(&mut self, method: &Method) -> Result<Self> {
    let builder = self.take_inner()?.method::<LibMethod>(method.into());
    Ok(Self::from(builder))
  }

  #[napi]
  pub fn get_method(&mut self) -> Result<Option<Method>> {
    Ok(self.get_inner()?.method_ref().cloned().map(Method::from))
  }

  #[napi]
  pub fn uri(&mut self, uri: String) -> Result<Self> {
    let builder = self.take_inner()?.uri::<String>(uri);
    Ok(Self::from(builder))
  }

  #[napi]
  pub fn get_uri(&mut self) -> Result<Option<String>> {
    Ok(self.get_inner()?.uri_ref().map(|uri| uri.to_string()))
  }

  #[napi]
  pub fn version(&mut self, version: &Version) -> Result<Self> {
    let builder = self.take_inner()?.version(version.into());
    Ok(Self::from(builder))
  }

  #[napi]
  pub fn get_version(&mut self) -> Result<Option<Version>> {
    let builder = self.get_inner()?;
    Ok(builder.version_ref().map(|v| Version::from(v.to_owned())))
  }

  #[napi]
  pub fn header(&mut self, key: String, value: String) -> Result<Self> {
    let builder = self.take_inner()?.header::<String, String>(key, value);
    Ok(Self::from(builder))
  }

  #[napi]
  pub fn get_headers(&mut self, env: Env) -> Result<Option<Object<'_>>> {
    let builder = self.get_inner()?;
    let Some(headers_map) = builder.headers_ref() else {
      return Ok(None);
    };
    let mut headers_obj = Object::new(&env)?;
    for (key, value) in headers_map {
      match value.to_str() {
        Ok(value) => headers_obj.set(key, value)?,
        Err(_) => headers_obj.set(key, Uint8Array::from(value.as_bytes()))?,
      }
    }
    Ok(Some(headers_obj))
  }

  #[napi]
  pub fn headers_mut(&mut self) {
    unimplemented!()
  }

  #[napi]
  pub fn extension(&mut self) {
    unimplemented!()
  }

  #[napi]
  pub fn get_extensions(&mut self) {
    unimplemented!()
  }

  #[napi]
  pub fn extensions_mut(&mut self) {
    unimplemented!()
  }

  #[napi]
  pub fn body(&mut self, body: &mut RequestBody) -> Result<Request> {
    let builder = self.take_inner()?;
    let body = body.take()?;
    let request: Box<dyn RequestInterface> = builder
      .body(body)
      .map(Box::new)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    Ok(Request::from(request))
  }
}
