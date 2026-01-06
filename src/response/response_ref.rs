use std::sync::{Arc, Mutex};

use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::response::{Response, cookie_options::CookieOptions, status_code::StatusCode};

#[napi]
#[derive(Clone, Debug)]
pub struct ResponseRef {
  inner: Arc<Mutex<Response>>,
}

impl From<Response> for ResponseRef {
  fn from(value: Response) -> Self {
    Self {
      inner: Arc::new(Mutex::new(value)),
    }
  }
}

#[napi]
impl ResponseRef {
  pub fn with_inner<F, T>(&self, f: F) -> Result<T>
  where
    F: FnOnce(&mut Response) -> Result<T>,
  {
    match self.inner.lock() {
      Ok(mut inner) => f(&mut inner),
      Err(e) => Err(Error::new(
        Status::GenericFailure,
        format!("Could not obtain lock on response. {e}"),
      )),
    }
  }

  #[napi]
  pub fn append(&mut self, field: String, value: Either<Vec<String>, String>) -> Result<()> {
    self.with_inner(|response| response.append(field, value))
  }

  #[napi]
  pub fn attachment(&mut self, file_path: Option<String>) -> Result<()> {
    self.with_inner(|response| response.attachment(file_path))
  }

  #[napi]
  pub fn clear_cookie(&mut self, name: String, options: Option<CookieOptions>) -> Result<()> {
    self.with_inner(|response| response.clear_cookie(name, options))
  }

  #[napi(js_name = "type")]
  pub fn typ(&mut self, typ: String) -> Result<()> {
    self.with_inner(|response| response.typ(typ))
  }

  #[napi]
  pub fn content_type(&mut self, typ: String) -> Result<()> {
    self.with_inner(|response| response.typ(typ))
  }

  #[napi]
  pub fn cookie(
    &mut self,
    name: String,
    value: String,
    options: Option<CookieOptions>,
  ) -> Result<()> {
    self.with_inner(|response| response.cookie(name, value, options))
  }

  #[napi]
  pub fn get(&mut self, field: String) -> Result<Either<String, Buffer>> {
    self.with_inner(|response| response.get(field))
  }

  #[napi]
  pub fn json(&mut self, body: Either5<String, i64, bool, Object, Null>, env: Env) -> Result<()> {
    self.with_inner(|response| response.json(body, env))
  }

  #[napi]
  pub fn send_status(&mut self, body: Either<u16, &StatusCode>, env: Env) -> Result<()> {
    self.with_inner(|response| response.send_status(body, env))
  }

  #[napi]
  pub fn send(
    &mut self,
    body: Either6<String, i64, bool, Object, Null, Buffer>,
    env: Env,
  ) -> Result<()> {
    self.with_inner(|response| response.send(body, env))
  }

  #[napi]
  pub fn status(&mut self, body: Either<u16, &StatusCode>) -> Result<()> {
    self.with_inner(|response| response.status(body))
  }
}
