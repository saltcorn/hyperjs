use std::fmt::Debug;

use bytes::Bytes;
use hyper::{body::Body as LibBody, Error as LibError};
use napi::bindgen_prelude::*;
use napi_derive::napi;

pub trait AppBody: LibBody<Data = Bytes, Error = LibError> + Debug + Send {}

pub type BoxedDynBody = Box<dyn AppBody>;

#[napi]
pub struct RequestBody {
  inner: Option<BoxedDynBody>,
}

impl RequestBody {
  pub fn take(&mut self) -> Result<BoxedDynBody> {
    match self.inner.take() {
      Some(body) => Ok(body),
      None => Err(Error::new(Status::GenericFailure, "Body is consumed.")),
    }
  }
}
