pub mod body_ref;
pub mod builder;
pub mod status;

use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::Response as LibResponse;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::version::Version;
use body_ref::ResponseBodyRef;
use builder::ResponseBuilder;
use status::StatusCode;

#[napi]
#[derive(Debug)]
pub struct Response {
  inner: LibResponse<Either<Full<Bytes>, Empty<Bytes>>>,
}

impl From<LibResponse<Either<Full<Bytes>, Empty<Bytes>>>> for Response {
  fn from(value: LibResponse<Either<Full<Bytes>, Empty<Bytes>>>) -> Self {
    Self { inner: value }
  }
}

impl Response {
  pub fn owned_inner(&self) -> LibResponse<Either<Full<Bytes>, Empty<Bytes>>> {
    self.inner.to_owned()
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
      Some(bytes) => Either::Left(Full::new(Bytes::copy_from_slice(bytes))),
      None => Either::Right(Empty::new()),
    };
    Ok(Response::from(LibResponse::new(body)))
  }

  #[napi(factory)]
  pub fn from_parts() {
    unimplemented!()
  }

  #[napi]
  pub fn status(&mut self) -> StatusCode {
    StatusCode::from(self.inner.status())
  }

  #[napi]
  pub fn version(&self) -> Version {
    Version::from(self.inner.version())
  }

  #[napi]
  pub fn headers(&self, env: Env) -> Result<Object<'_>> {
    let mut headers_obj = Object::new(&env)?;
    for (key, value) in self.inner.headers() {
      match value.to_str() {
        Ok(value) => headers_obj.set(key, value)?,
        Err(_) => headers_obj.set(key, Uint8Array::from(value.as_bytes()))?,
      }
    }
    Ok(headers_obj)
  }

  #[napi]
  pub fn body(&self, request: Reference<Response>, env: Env) -> Result<ResponseBodyRef> {
    let shared_body_ref = request.share_with(env, |request| Ok(request.inner.body()))?;
    Ok(ResponseBodyRef::new(shared_body_ref))
  }
}
