use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Request, WrappedRequest};

#[napi]
impl Request {
  #[napi(getter)]
  pub fn params(&self, env: Env) -> Result<Object<'_>> {
    self.with_inner(|request| request.params(env))
  }
}

impl WrappedRequest {
  pub fn params(&self, env: Env) -> Result<Object<'static>> {
    let mut headers_obj = Object::new(&env)?;
    for (key, value) in &self.params {
      headers_obj.set(key, value)?;
    }
    Ok(headers_obj)
  }
}
