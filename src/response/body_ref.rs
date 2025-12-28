use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::response::Response;

#[napi]
pub struct ResponseBodyRef {
  inner: SharedReference<Response, &'static Either<Full<Bytes>, Empty<Bytes>>>,
}

impl ResponseBodyRef {
  pub fn new(inner: SharedReference<Response, &'static Either<Full<Bytes>, Empty<Bytes>>>) -> Self {
    Self { inner }
  }
}
