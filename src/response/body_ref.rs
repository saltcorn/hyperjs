use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::Error as LibError;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::response::Response;

#[napi]
pub struct ResponseBodyRef {
  inner: SharedReference<Response, &'static BoxBody<Bytes, LibError>>,
}

impl ResponseBodyRef {
  pub fn new(inner: SharedReference<Response, &'static BoxBody<Bytes, LibError>>) -> Self {
    Self { inner }
  }
}
