use std::convert::Infallible;

pub enum Error {
  Hyper(hyper::Error),
  Infallible,
}

impl From<hyper::Error> for Error {
  fn from(value: hyper::Error) -> Self {
    Self::Hyper(value)
  }
}

impl From<Infallible> for Error {
  fn from(_value: Infallible) -> Self {
    Self::Infallible
  }
}
