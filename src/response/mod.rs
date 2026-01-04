mod append;
mod attachment;
pub mod body_ref;
pub mod builder;
mod content_type;
mod cookie;
mod get;
pub mod status;

use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::{Error as LibError, Response as LibResponse};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{utilities::empty, version::Version};
use body_ref::ResponseBodyRef;
use status::StatusCode;

type ResponseInner = LibResponse<BoxBody<Bytes, LibError>>;

#[napi]
#[derive(Debug, Default)]
pub struct Response {
  inner: Option<ResponseInner>,
}

impl From<ResponseInner> for Response {
  fn from(value: ResponseInner) -> Self {
    Self { inner: Some(value) }
  }
}

impl Response {
  fn inner(&self) -> Result<&ResponseInner> {
    self.inner.as_ref().ok_or(Error::new(
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

  pub fn unwrap_inner_or_default(&mut self) -> ResponseInner {
    self.inner.take().unwrap_or(ResponseInner::new(empty()))
  }

  pub fn set_inner(&mut self, inner: ResponseInner) -> Result<()> {
    if self.inner.is_some() {
      return Err(Error::new(
        Status::GenericFailure,
        "Unexpected response overwrite.",
      ));
    }
    self.inner = Some(inner);
    Ok(())
  }
}

#[napi]
impl Response {
  #[napi(constructor)]
  pub fn new() -> Response {
    Self::default()
  }

  #[napi]
  pub fn status(&mut self) -> Result<StatusCode> {
    Ok(StatusCode::from(self.inner()?.status()))
  }

  #[napi]
  pub fn version(&self) -> Result<Version> {
    Ok(Version::from(self.inner()?.version()))
  }

  #[napi]
  pub fn headers(&self, env: Env) -> Result<Object<'_>> {
    let mut headers_obj = Object::new(&env)?;
    let headers_map = self.inner()?.headers();
    for key in headers_map.keys() {
      let mut header_values = Vec::new();
      for value in headers_map.get_all(key) {
        match value.to_str() {
          Ok(value) => header_values.push(value),
          Err(_) => {
            headers_obj.set(key, Uint8Array::from(value.as_bytes()))?;
            continue;
          }
        }
      }
      if !header_values.is_empty() {
        headers_obj.set(key, header_values.join(", "))?
      }
    }
    Ok(headers_obj)
  }

  #[napi]
  pub fn body(&self, request: Reference<Response>, env: Env) -> Result<ResponseBodyRef> {
    let shared_body_ref = request.share_with(env, |request| Ok(request.inner()?.body()))?;
    Ok(ResponseBodyRef::new(shared_body_ref))
  }
}
