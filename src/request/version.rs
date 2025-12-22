use hyper::http::version::Version as LibVersion;
use napi_derive::napi;

#[napi]
pub struct Version {
  inner: LibVersion,
}

impl From<LibVersion> for Version {
  fn from(value: LibVersion) -> Self {
    Self { inner: value }
  }
}

impl From<&Version> for LibVersion {
  fn from(value: &Version) -> Self {
    value.inner.to_owned()
  }
}

#[napi]
impl Version {
  #[napi(factory)]
  pub fn http_09() -> Self {
    Self::from(LibVersion::HTTP_09)
  }

  #[napi(factory)]
  pub fn http_10() -> Self {
    Self::from(LibVersion::HTTP_10)
  }

  #[napi(factory)]
  pub fn http_11() -> Self {
    Self::from(LibVersion::HTTP_11)
  }

  #[napi(factory)]
  pub fn http_2() -> Self {
    Self::from(LibVersion::HTTP_2)
  }

  #[napi(factory)]
  pub fn http_3() -> Self {
    Self::from(LibVersion::HTTP_3)
  }
}
