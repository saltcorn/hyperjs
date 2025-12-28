use bytes::Bytes;
use hyper::{
  body::{Body as LibBody, Incoming},
  Error as LibError, Request as LibRequest,
};
use napi::bindgen_prelude::*;

use super::{body::BoxedDynBody, method::Method};
use crate::version::Version;

pub trait RequestInterface: std::fmt::Debug + Send {
  fn method(&self) -> Method;

  fn uri(&self) -> String;

  fn version(&self) -> Version;

  fn headers(&self, env: &Env) -> Result<Object<'_>>;

  fn body(&self) -> &dyn LibBody<Data = Bytes, Error = LibError>;
}

impl RequestInterface for LibRequest<BoxedDynBody> {
  fn method(&self) -> Method {
    todo!()
  }

  fn uri(&self) -> String {
    todo!()
  }

  fn version(&self) -> Version {
    todo!()
  }

  fn headers(&self, env: &Env) -> Result<Object<'_>> {
    todo!()
  }

  fn body(&self) -> &dyn LibBody<Data = Bytes, Error = LibError> {
    todo!()
  }
}

impl RequestInterface for LibRequest<Incoming> {
  fn method(&self) -> Method {
    todo!()
  }

  fn uri(&self) -> String {
    todo!()
  }

  fn version(&self) -> Version {
    todo!()
  }

  fn headers(&self, env: &Env) -> Result<Object<'_>> {
    todo!()
  }

  fn body(&self) -> &dyn LibBody<Data = Bytes, Error = LibError> {
    todo!()
  }
}
