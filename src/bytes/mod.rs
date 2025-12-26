use bytes::{Buf, Bytes as LibBytes};
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub struct Bytes {
  inner: LibBytes,
}

impl From<LibBytes> for Bytes {
  fn from(value: LibBytes) -> Self {
    Self { inner: value }
  }
}

impl Default for Bytes {
  fn default() -> Self {
    Self::from(LibBytes::new())
  }
}

impl Bytes {
  pub fn owned_inner(&self) -> LibBytes {
    self.inner.to_owned()
  }
}

#[napi]
impl Bytes {
  #[napi(constructor)]
  pub fn new() -> Self {
    Self::default()
  }

  #[napi]
  pub fn from_owner(owner: Buffer) -> Self {
    Self::from(LibBytes::from_owner(owner))
  }

  #[napi(getter, js_name = "length")]
  pub fn len(&mut self) -> u32 {
    self.inner.len() as u32
  }

  #[napi]
  pub fn is_empty(&mut self) -> bool {
    self.inner.is_empty()
  }

  #[napi]
  pub fn is_unique(&mut self) -> bool {
    self.inner.is_unique()
  }

  #[napi(factory)]
  pub fn copy_from_slice(data: &[u8]) -> Self {
    Self::from(LibBytes::copy_from_slice(data))
  }

  #[napi]
  pub fn slice(&mut self, start_idx: u32, end_idx: u32) -> Self {
    let start_idx = start_idx as usize;
    let end_idx = end_idx as usize;
    Self::from(self.inner.slice(start_idx..end_idx))
  }

  #[napi]
  pub fn slice_ref(&mut self, subset: &[u8]) -> Self {
    Self::from(self.inner.slice_ref(subset))
  }

  #[napi]
  pub fn split_off(&mut self, mid: u32) -> Self {
    Self::from(self.inner.split_off(mid as usize))
  }

  #[napi]
  pub fn split_to(&mut self, at: u32) -> Self {
    Self::from(self.inner.split_to(at as usize))
  }

  #[napi]
  pub fn truncate(&mut self, len: u32) {
    self.inner.truncate(len as usize);
  }

  #[napi]
  pub fn clear(&mut self) {
    self.inner.clear();
  }
}

impl Buf for Bytes {
  fn remaining(&self) -> usize {
    self.inner.remaining()
  }

  fn chunk(&self) -> &[u8] {
    self.inner.chunk()
  }

  fn advance(&mut self, cnt: usize) {
    self.inner.advance(cnt);
  }
}
