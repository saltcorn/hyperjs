use std::time::SystemTime;

use napi::{JsDate, bindgen_prelude::*};
use napi_derive::napi;

use crate::utilities;

#[napi(object)]
#[derive(Default, Clone)]
pub struct ClearCookie {}

#[napi(object)]
#[derive(Default, Clone)]
pub struct CookieOptions {
  pub domain: Option<String>,
  pub encode: Option<Function<'static, String, String>>,
  pub expires: Option<Either<JsDate<'static>, ClearCookie>>,
  pub http_only: Option<bool>,
  pub max_age: Option<i64>,
  pub path: Option<String>,
  pub partitioned: Option<bool>,
  pub priority: Option<String>,
  pub secure: Option<bool>,
  pub signed: Option<bool>,
  pub same_site: Option<Either<bool, String>>,
}

impl CookieOptions {
  pub fn mark_for_clearing(&mut self) {
    self.expires = Some(Either::B(ClearCookie {}))
  }

  pub fn get_pairs_as_strings(&self) -> Result<Vec<String>> {
    let mut options_strings = Vec::new();

    if let Some(expires) = &self.expires {
      match expires {
        Either::A(expires) => {
          let expires = httpdate::fmt_http_date(utilities::js_date_to_system_time(expires)?);
          options_strings.push(format!("Expires={expires}"));
        }
        Either::B(_) => {
          let expires = httpdate::fmt_http_date(SystemTime::UNIX_EPOCH);
          options_strings.push(format!("Expires={expires}"));
        }
      }
    }

    if let Some(max_age) = self.max_age {
      options_strings.push(format!("Max-Age={max_age}"));
    }

    if let Some(domain) = &self.domain {
      options_strings.push(format!("Domain={domain}"));
    }

    match &self.path {
      Some(path) => {
        options_strings.push(format!("Path={path}"));
      }
      None => {
        options_strings.push("Path=/".to_owned());
      }
    }

    if let Some(secure) = self.secure
      && secure
    {
      options_strings.push("Secure".to_owned());
    }

    if let Some(http_only) = self.http_only
      && http_only
    {
      options_strings.push("HttpOnly".to_owned());
    }

    Ok(options_strings)
  }
}
