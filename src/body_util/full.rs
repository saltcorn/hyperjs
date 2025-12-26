use std::ops::{Deref, DerefMut};

use bytes::Bytes as LibBytes;
use http_body_util::Full as LibFull;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::bytes::Bytes;

#[napi]
pub struct Full {
  inner: LibFull<LibBytes>,
}

impl From<LibFull<LibBytes>> for Full {
  fn from(value: LibFull<LibBytes>) -> Self {
    Self { inner: value }
  }
}

#[napi]
impl Full {
  #[napi(constructor)]
  pub fn new(data: Buffer) -> Result<Self> {
    Ok(Self::from(LibFull::new(LibBytes::from_owner(data))))
  }

  #[napi(factory)]
  pub fn from_bytes(data: &Bytes) -> Result<Self> {
    Ok(Self::from(LibFull::new(data.owned_inner())))
  }
}
