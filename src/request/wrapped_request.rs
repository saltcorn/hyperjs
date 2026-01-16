use std::collections::HashMap;

use hyper::{Request as HyperRequest, body::Incoming as IncomingBody};
use napi::bindgen_prelude::*;
use serde_json::Value as JsonValue;

pub struct WrappedRequest {
  pub(super) inner: Option<HyperRequest<IncomingBody>>,
  pub(super) params: HashMap<String, String>,
  pub(super) body: Option<Either<String, JsonValue>>,
}

impl From<HyperRequest<IncomingBody>> for WrappedRequest {
  fn from(value: HyperRequest<IncomingBody>) -> Self {
    Self {
      inner: Some(value),
      params: HashMap::with_capacity(0),
      body: None,
    }
  }
}

impl WrappedRequest {
  pub fn inner_mut(&mut self) -> Result<&mut HyperRequest<IncomingBody>> {
    self
      .inner
      .as_mut()
      .ok_or(Error::new(Status::GenericFailure, "Body already parsed."))
  }

  pub fn inner(&self) -> Result<&HyperRequest<IncomingBody>> {
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

  pub fn take_inner(&mut self) -> Result<HyperRequest<IncomingBody>> {
    self.inner.take().ok_or(Error::new(
      Status::GenericFailure,
      "Method called on consumed Request.",
    ))
  }

  pub fn set_body(&mut self, body: Either<String, JsonValue>) {
    self.body = Some(body)
  }
}
