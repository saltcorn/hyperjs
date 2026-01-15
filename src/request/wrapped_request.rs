use std::collections::HashMap;

use http_body_util::{BodyDataStream, BodyExt};
use hyper::{Request as HyperRequest, body::Incoming as IncomingBody};
use napi::bindgen_prelude::*;

pub struct WrappedRequest {
  pub(super) inner: Option<HyperRequest<IncomingBody>>,
  pub(super) params: HashMap<String, String>,
  body: Option<Either<String, ObjectRef>>,
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

  pub fn body(&mut self) -> Result<BodyDataStream<IncomingBody>> {
    let body_stream = self
      .inner
      .take()
      .ok_or(Error::new(
        Status::GenericFailure,
        "Method called on consumed Request.",
      ))?
      .into_body()
      .into_data_stream();
    Ok(body_stream)
  }

  pub fn set_body(&mut self, body: Either<String, ObjectRef>) {
    self.body = Some(body)
  }
}
