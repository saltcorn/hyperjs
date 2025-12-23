use hyper::Response as LibResponse;
use napi::bindgen_prelude::*;

use super::status::StatusCode;
use crate::{
  body::{Body, SupportedBodies},
  version::Version,
};

pub enum BodyResponse {
  Empty(LibResponse<()>),
  String(LibResponse<String>),
}

impl From<LibResponse<()>> for BodyResponse {
  fn from(value: LibResponse<()>) -> Self {
    BodyResponse::Empty(value)
  }
}

impl From<LibResponse<String>> for BodyResponse {
  fn from(value: LibResponse<String>) -> Self {
    BodyResponse::String(value)
  }
}

impl From<&Body> for BodyResponse {
  fn from(value: &Body) -> Self {
    match value.inner() {
      SupportedBodies::Empty => LibResponse::new(()).into(),
      SupportedBodies::String(body) => LibResponse::new(body.to_owned()).into(),
    }
  }
}

impl BodyResponse {
  pub fn status(&self) -> StatusCode {
    match self {
      BodyResponse::Empty(r) => r.status(),
      BodyResponse::String(r) => r.status(),
    }
    .to_owned()
    .into()
  }

  pub fn version(&self) -> Version {
    match self {
      BodyResponse::Empty(r) => r.version(),
      BodyResponse::String(r) => r.version(),
    }
    .to_owned()
    .into()
  }

  pub fn headers(&self, env: &Env) -> Result<Object<'_>> {
    let headers_map = match self {
      BodyResponse::Empty(r) => r.headers(),
      BodyResponse::String(r) => r.headers(),
    };
    let mut headers_obj = Object::new(env)?;
    for (key, value) in headers_map {
      match value.to_str() {
        Ok(value) => headers_obj.set(key, value)?,
        Err(_) => headers_obj.set(key, Uint8Array::from(value.as_bytes()))?,
      }
    }
    Ok(headers_obj)
  }

  pub fn extensions(&self) {
    unimplemented!()
  }

  pub fn body(&self) -> Body {
    match self {
      BodyResponse::Empty(_r) => Body::empty(),
      BodyResponse::String(r) => Body::string(r.body().to_owned()),
    }
  }
}
