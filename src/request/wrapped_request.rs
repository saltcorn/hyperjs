use std::collections::HashMap;

use hyper::{Request as HyperRequest, body::Incoming as IncomingBody};

#[derive(Debug)]
pub struct WrappedRequest {
  pub(super) inner: HyperRequest<IncomingBody>,
  pub(super) params: HashMap<String, String>,
}

impl From<HyperRequest<IncomingBody>> for WrappedRequest {
  fn from(value: HyperRequest<IncomingBody>) -> Self {
    Self {
      inner: value,
      params: HashMap::with_capacity(0),
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
}
