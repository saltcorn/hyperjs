use hyper::header::{HeaderValue, SET_COOKIE};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Response, cookie_options::CookieOptions};

#[napi]
impl Response {
  /// Clears the cookie with the specified name by sending a `Set-Cookie` header that sets its expiration date in the past.
  /// This instructs the client that the cookie has expired and is no longer valid. For more information about available
  /// options, see `res.cookie()`.
  ///
  /// > The expires and max-age options are being ignored completely.
  ///
  /// > Web browsers and other compliant clients will only clear the cookie if the given options is identical to those given
  /// > to res.cookie()
  ///
  /// ```javascript
  /// res.cookie('name', 'tobi', { path: '/admin' })
  /// res.clearCookie('name', { path: '/admin' })
  /// ```
  #[napi]
  pub fn clear_cookie(&mut self, name: String, mut options: Option<CookieOptions>) -> Result<()> {
    let mut inner = self.unwrap_inner_or_default();

    let mut option_string = String::new();

    if let Some(options) = &mut options {
      options.mark_for_clearing();

      let options_strings = options.get_pairs_as_strings()?;

      if !options_strings.is_empty() {
        option_string = options_strings.join("; ")
      }
    }

    let cookie_string = [format!("{name}="), option_string].join("; ");

    let header_value = HeaderValue::from_str(&cookie_string)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    inner.headers_mut().append(SET_COOKIE, header_value);

    self.set_inner(inner)
  }
}

#[cfg(test)]
mod tests {
  use hyper::header::{GetAll, HeaderValue, SET_COOKIE};

  use super::{CookieOptions, Response};

  fn assert_exists(header_values: &GetAll<'_, HeaderValue>, value: &str) {
    assert!(header_values.iter().any(|v| v.to_str().unwrap() == value))
  }

  #[test]
  fn clear_cookie() {
    let mut res = Response::new();

    let options = CookieOptions {
      path: Some("/".to_owned()),
      domain: Some("example.com".to_owned()),
      ..Default::default()
    };
    res.clear_cookie("lang".to_owned(), Some(options)).unwrap();

    let inner = res.inner().unwrap();
    let header_values = inner.headers().get_all(SET_COOKIE.as_str());

    assert_exists(
      &header_values,
      "lang=; Expires=Thu, 01 Jan 1970 00:00:00 GMT; Domain=example.com; Path=/",
    );
  }
}
