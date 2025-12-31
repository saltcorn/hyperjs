pub mod body_ref;
pub mod builder;
pub mod status;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{Error as LibError, Response as LibResponse};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{
  utilities::{empty, full},
  version::Version,
};
use body_ref::ResponseBodyRef;
use builder::ResponseBuilder;
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
}

#[napi]
impl Response {
  #[napi]
  pub fn builder() -> ResponseBuilder {
    ResponseBuilder::default()
  }

  #[napi(constructor)]
  pub fn new(&mut self, body: Option<&[u8]>) -> Result<Response> {
    let body = match body {
      Some(bytes) => full(Bytes::copy_from_slice(bytes)).boxed(),
      None => empty().boxed(),
    };
    Ok(Response::from(LibResponse::new(body)))
  }

  #[napi(factory)]
  pub fn from_parts() {
    unimplemented!()
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
    for (key, value) in self.inner()?.headers() {
      match value.to_str() {
        Ok(value) => headers_obj.set(key, value)?,
        Err(_) => headers_obj.set(key, Uint8Array::from(value.as_bytes()))?,
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
