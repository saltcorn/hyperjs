use std::{
  pin::Pin,
  task::{Poll, ready},
};

use bytes::Bytes;
use http_body::Body as HttpBody;
use http_body_util::Full;
use hyper::Response as LibResponse;
use hyper_staticfile::Body as StaticFileBody;
use napi::{Error, Result, Status};

use crate::utilities::full;

pub enum CrateBody {
  Empty,
  Full(Full<Bytes>),
  StaticFile(StaticFileBody),
}

impl HttpBody for CrateBody {
  type Data = Bytes;

  type Error = std::io::Error;

  fn poll_frame(
    mut self: std::pin::Pin<&mut Self>,
    cx: &mut std::task::Context<'_>,
  ) -> std::task::Poll<Option<std::result::Result<http_body::Frame<Self::Data>, Self::Error>>> {
    let opt = ready!(match *self {
      Self::Empty => return Poll::Ready(None),
      Self::Full(ref mut stream) => Pin::new(stream)
        .poll_frame(cx)
        .map(|s| s.map(|s| s.map_err(|never| match never {}))),
      Self::StaticFile(ref mut stream) => Pin::new(stream).poll_frame(cx),
    });
    Poll::Ready(opt)
  }
}

impl From<StaticFileBody> for CrateBody {
  fn from(value: StaticFileBody) -> Self {
    Self::StaticFile(value)
  }
}

type ResponseInner = LibResponse<CrateBody>;

pub struct WrappedResponse {
  inner: Option<ResponseInner>,
}

impl Default for WrappedResponse {
  fn default() -> Self {
    Self {
      inner: Some(LibResponse::new(CrateBody::Empty)),
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

  pub fn set_inner(&mut self, inner: ResponseInner) -> &mut ResponseInner {
    self.inner.insert(inner)
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
      None => self.take()?.map(|_| CrateBody::Empty),
    };
    self.inner = Some(response);
    Ok(())
  }
}
