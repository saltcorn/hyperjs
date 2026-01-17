use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::{Error as LibError, Response as LibResponse};
use napi::{Error, Result, Status};

use crate::utilities::{empty, full};

type ResponseInner = LibResponse<BoxBody<Bytes, LibError>>;

#[derive(Debug)]
pub struct WrappedResponse {
  inner: Option<ResponseInner>,
}

impl Default for WrappedResponse {
  fn default() -> Self {
    Self {
      inner: Some(LibResponse::new(empty())),
    }
  }
}

impl From<ResponseInner> for WrappedResponse {
  fn from(value: ResponseInner) -> Self {
    Self { inner: Some(value) }
  }
}

impl WrappedResponse {
  pub fn inner(&mut self) -> Result<&mut ResponseInner> {
    self.inner.as_mut().ok_or(Error::new(
      Status::GenericFailure,
      "Misuse of consumed response.",
    ))
  }

  pub fn take(&mut self) -> Result<ResponseInner> {
    self.inner.take().ok_or(Error::new(
      Status::GenericFailure,
      "Misuse of consumed response.",
    ))
  }

  pub fn end(&mut self, data: Option<Bytes>) -> Result<()> {
    let response = match data {
      Some(data) => self.take()?.map(|_| full(data)),
      None => self.take()?.map(|_| empty()),
    };
    self.inner = Some(response);
    Ok(())
  }
}
