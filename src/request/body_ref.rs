use bytes::Bytes;
use hyper::{body::Body as LibBody, Error as LibError};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::request::Request;

#[napi]
pub struct RequestBodyRef {
  inner: SharedReference<Request, &'static dyn LibBody<Data = Bytes, Error = LibError>>,
}

impl RequestBodyRef {
  pub fn new(
    inner: SharedReference<Request, &'static dyn LibBody<Data = Bytes, Error = LibError>>,
  ) -> Self {
    Self { inner }
  }
}
