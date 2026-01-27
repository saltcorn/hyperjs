use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Request, WrappedRequest};

#[napi]
impl Request {
  /// Contains a string corresponding to the HTTP method of the request: `GET`,
  /// `POST`, `PUT`, and so on.
  #[napi(getter)]
  pub fn method(&self) -> Result<String> {
    self.with_inner(|request| request.method())
  }
}

impl WrappedRequest {
  pub fn method(&self) -> Result<String> {
    Ok(self.inner()?.method().as_str().to_owned())
  }
}
