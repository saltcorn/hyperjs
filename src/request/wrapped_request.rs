use std::collections::HashMap;

use bytes::Bytes;
use http_body_util::{BodyExt, combinators::BoxBody};
use hyper::{Error as LibError, Request as HyperRequest, body::Body};
use napi::bindgen_prelude::*;
use serde_json::Value as JsonValue;

use crate::utilities;

type RequestInner = HyperRequest<BoxBody<Bytes, Box<dyn std::error::Error + Sync + Send>>>;

#[derive(Debug)]
pub struct WrappedRequest {
  pub(super) inner: Option<RequestInner>,
  pub(super) params: HashMap<String, String>,
  pub(super) body: Option<Either3<String, JsonValue, Vec<u8>>>,
}

impl Default for WrappedRequest {
  fn default() -> Self {
    Self::from(HyperRequest::new(utilities::empty()))
  }
}

impl<T: BodyExt + Body<Data = Bytes, Error = LibError> + Send + Sync + 'static>
  From<HyperRequest<T>> for WrappedRequest
{
  fn from(value: HyperRequest<T>) -> Self {
    let request = value.map(|body| {
      body
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        .boxed()
    });
    Self {
      inner: Some(request),
      params: HashMap::with_capacity(0),
      body: None,
    }
  }
}

impl WrappedRequest {
  pub fn inner_mut(&mut self) -> Result<&mut RequestInner> {
    self
      .inner
      .as_mut()
      .ok_or(Error::new(Status::GenericFailure, "Body already parsed."))
  }

  pub fn set_inner(&mut self, inner: RequestInner) -> &mut RequestInner {
    self.inner.insert(inner)
  }

  pub fn inner(&self) -> Result<&RequestInner> {
    self
      .inner
      .as_ref()
      .ok_or(Error::new(Status::GenericFailure, "Body already parsed."))
  }

  pub fn set_param(&mut self, k: String, v: String) {
    self.params.insert(k, v);
  }

  pub fn set_params<I, K, V>(&mut self, iterator: I)
  where
    I: Iterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
  {
    for (k, v) in iterator {
      self.params.insert(k.into(), v.into());
    }
  }

  pub fn take_inner(&mut self) -> Result<RequestInner> {
    self.inner.take().ok_or(Error::new(
      Status::GenericFailure,
      "Method called on consumed Request.",
    ))
  }

  pub fn set_body(&mut self, body: Either3<String, JsonValue, Vec<u8>>) {
    self.body = Some(body)
  }
}
