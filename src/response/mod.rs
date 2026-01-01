pub mod body_ref;
pub mod builder;
pub mod status;

use std::str::FromStr;

use bytes::Bytes;
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
  header::{HeaderName, HeaderValue},
  Error as LibError, Response as LibResponse,
};
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

  /// Appends the specified value to the HTTP response header field. If the header is not already set, it creates the header
  /// with the specified value. The value parameter can be a string or an array.
  /// > **&#10155; Note**
  /// >
  /// > calling `res.set()` after `res.append()` will reset the previously-set header value.
  ///
  /// ```javascript
  /// res.append('Link', ['<http://localhost/>', '<http://localhost:3000/>'])
  /// res.append('Set-Cookie', 'foo=bar; Path=/; HttpOnly')
  /// res.append('Warning', '199 Miscellaneous warning')
  /// ```
  #[napi]
  pub fn append(&mut self, field: String, value: Either<Vec<String>, String>) -> Result<()> {
    let mut inner = self.unwrap_inner_or_default();
    let headers_map = inner.headers_mut();
    let header_name =
      HeaderName::from_str(&field).map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?;
    match value {
      Either::A(values) => {
        for value in values.iter() {
          let header_value = HeaderValue::from_str(value)
            .map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?;
          headers_map.append(&header_name, header_value);
        }
      }
      Either::B(value) => {
        let header_value = HeaderValue::from_str(&value)
          .map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?;
        headers_map.append(header_name, header_value);
      }
    }
    self.set_inner(inner)
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
          Err(_) => headers_obj.set(key, Uint8Array::from(value.as_bytes()))?,
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

#[cfg(test)]
mod tests {
  use napi::Either;

  use super::Response;

  #[test]
  fn append() {
    let mut res = Response::new();
    res
      .append(
        "Link".to_owned(),
        Either::A(vec![
          "<http://localhost/>".to_owned(),
          "<http://localhost:3000/>".to_owned(),
        ]),
      )
      .unwrap();
    res
      .append(
        "Set-Cookie".to_owned(),
        Either::B("foo=bar; Path=/; HttpOnly".to_owned()),
      )
      .unwrap();
    res
      .append(
        "Warning".to_owned(),
        Either::B("199 Miscellaneous warning".to_owned()),
      )
      .unwrap();
  }
}
