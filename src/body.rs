use napi_derive::napi;

#[derive(Debug, Clone)]
pub enum SupportedBodies {
  Empty,
  String(String),
}

#[napi]
pub struct Body {
  inner: SupportedBodies,
}

impl From<SupportedBodies> for Body {
  fn from(value: SupportedBodies) -> Self {
    Self { inner: value }
  }
}

impl Body {
  pub fn inner(&self) -> &SupportedBodies {
    &self.inner
  }
}

#[napi]
impl Body {
  #[napi(factory)]
  pub fn empty() -> Self {
    Self::from(SupportedBodies::Empty)
  }

  #[napi(factory)]
  pub fn string(body: String) -> Self {
    Self::from(SupportedBodies::String(body))
  }
}
