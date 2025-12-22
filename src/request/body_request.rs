use hyper::Request as LibRequest;
use napi::bindgen_prelude::*;

use super::{body::Body, method::Method, version::Version};

pub enum BodyRequest {
  Empty(LibRequest<()>),
  String(LibRequest<String>),
}

impl BodyRequest {
  pub fn method(&self) -> Method {
    match self {
      BodyRequest::Empty(r) => r.method(),
      BodyRequest::String(r) => r.method(),
    }
    .to_owned()
    .into()
  }

  pub fn uri(&self) -> String {
    match self {
      BodyRequest::Empty(r) => r.uri(),
      BodyRequest::String(r) => r.uri(),
    }
    .to_string()
  }

  pub fn version(&self) -> Version {
    match self {
      BodyRequest::Empty(r) => r.version(),
      BodyRequest::String(r) => r.version(),
    }
    .to_owned()
    .into()
  }

  pub fn headers(&self, env: &Env) -> Result<Object<'_>> {
    let headers_map = match self {
      BodyRequest::Empty(r) => r.headers(),
      BodyRequest::String(r) => r.headers(),
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

  pub fn body(&self) -> Body {
    match self {
      BodyRequest::Empty(_r) => Body::empty(),
      BodyRequest::String(r) => Body::string(r.body().to_owned()),
    }
  }
}
