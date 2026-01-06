use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::response::Response;

#[napi]
pub struct ResponseRef {
  inner: SharedReference<Response, &'static Response>,
}

impl ResponseRef {
  pub fn new(inner: SharedReference<Response, &'static Response>) -> Self {
    Self { inner }
  }
}
