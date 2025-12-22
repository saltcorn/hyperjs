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

#[napi]
impl Version {
  #[napi(constructor)]
  pub fn http_09() -> Self {
    Self::from(LibVersion::HTTP_09)
  }

  #[napi(constructor)]
  pub fn http_10() -> Self {
    Self::from(LibVersion::HTTP_10)
  }

  #[napi(constructor)]
  pub fn http_11() -> Self {
    Self::from(LibVersion::HTTP_11)
  }

  #[napi(constructor)]
  pub fn http_2() -> Self {
    Self::from(LibVersion::HTTP_2)
  }

  #[napi(constructor)]
  pub fn http_3() -> Self {
    Self::from(LibVersion::HTTP_3)
  }
}
