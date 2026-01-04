use hyper::header::{HeaderValue, SET_COOKIE};
use napi::{JsDate, bindgen_prelude::*};
use napi_derive::napi;

use crate::utilities;

use super::Response;

#[napi(object)]
#[derive(Default, Clone)]
pub struct CookieOptions {
  pub domain: Option<String>,
  pub encode: Option<Function<'static, String, String>>,
  pub expires: Option<JsDate<'static>>,
  pub http_only: Option<bool>,
  pub max_age: Option<i64>,
  pub path: Option<String>,
  pub partitioned: Option<bool>,
  pub priority: Option<String>,
  pub secure: Option<bool>,
  pub signed: Option<bool>,
  pub same_site: Option<Either<bool, String>>,
}

#[napi]
impl Response {
  /// Sets cookie `name` to `value`. The `value` parameter may be a string or object converted to JSON.
  ///
  /// The `options` parameter is an object that can have the following properties.
  ///
  /// | Property      | Type     | Description |
  /// | --------      | -----    | --- |
  /// | `domain`      | String   | Domain name for the cookie. Defaults to the domain name of the app. |
  /// | `encode`      | Function | A synchronous function used for cookie value encoding. Defaults to `encodeURIComponent`. |
  /// | `expires`     | Date     | Expiry date of the cookie in GMT. If not specified or set to 0, creates a session cookie. |
  /// | `httpOnly`    | Boolean  | Flags the cookie to be accessible only by the web server. |
  /// | `maxAge`      | Number   | Convenient option for setting the expiry time relative to the current time in milliseconds. |
  /// | `path`        | String   | Path for the cookie. Defaults to "/". |
  /// | `partitioned` | Boolean  | Indicates that the cookie should be stored using partitioned storage. See [Cookies Having Independent Partitioned State (CHIPS)](https://developer.mozilla.org/en-US/docs/Web/Privacy/Partitioned_cookies) for more details. |
  /// | `priority`    | String   | Value of the "Priority" Set-Cookie attribute. |
  /// | `secure`      | Boolean  | Marks the cookie to be used with HTTPS only. |
  /// | `signed`      | Boolean  | Indicates if the cookie should be signed. |
  /// | `sameSite`    | Boolean or String | Value of the "SameSite" Set-Cookie attribute. More information at https://tools.ietf.org/html/draft-ietf-httpbis-cookie-same-site-00#section-4.1.1. |
  ///
  /// > All `res.cookie()` does is set the HTTP `Set-Cookie` header with the options provided. Any option not specified defaults to the value stated in [RFC 6265](http://tools.ietf.org/html/rfc6265).
  ///
  /// For example:
  ///
  /// ```javascript
  /// res.cookie('name', 'tobi', { domain: '.example.com', path: '/admin', secure: true })
  /// res.cookie('rememberme', '1', { expires: new Date(Date.now() + 900000), httpOnly: true })
  /// ```
  ///
  /// You can set multiple cookies in a single response by calling res.cookie multiple times, for example:
  ///
  /// ```javascript
  /// res
  ///   .status(201)
  ///   .cookie('access_token', `Bearer ${token}`, {
  ///     expires: new Date(Date.now() + 8 * 3600000) // cookie will be removed after 8 hours
  ///   })
  ///   .cookie('test', 'test')
  ///   .redirect(301, '/admin')
  /// ```
  ///
  /// The encode option allows you to choose the function used for cookie value encoding. Does not support asynchronous functions.
  ///
  /// Example use case: You need to set a domain-wide cookie for another site in your organization. This other site (not under your administrative control) does not use URI-encoded cookie values.
  ///
  /// ```javascript
  /// // Default encoding
  /// res.cookie('some_cross_domain_cookie', 'http://mysubdomain.example.com', { domain: 'example.com' })
  /// // Result: 'some_cross_domain_cookie=http%3A%2F%2Fmysubdomain.example.com; Domain=example.com; Path=/'
  ///
  /// // Custom encoding
  /// res.cookie('some_cross_domain_cookie', 'http://mysubdomain.example.com', { domain: 'example.com', encode: String })
  /// // Result: 'some_cross_domain_cookie=http://mysubdomain.example.com; Domain=example.com; Path=/;'
  /// ```
  ///
  /// The maxAge option is a convenience option for setting "expires" relative to the current time in milliseconds. The following is equivalent to the second example above.
  ///
  /// ```javascript
  /// res.cookie('rememberme', '1', { maxAge: 900000, httpOnly: true })
  /// ```
  ///
  /// You can pass an object as the value parameter; it is then serialized as JSON and parsed by bodyParser() middleware.
  ///
  /// ```javascript
  /// res.cookie('cart', { items: [1, 2, 3] })
  /// res.cookie('cart', { items: [1, 2, 3] }, { maxAge: 900000 })
  /// ```
  ///
  /// When using cookie-parser middleware, this method also supports signed cookies. Simply include the signed option set to true. Then, res.cookie() will use the secret passed to cookieParser(secret) to sign the value.
  ///
  /// ```javascript
  /// res.cookie('name', 'tobi', { signed: true })
  /// ```
  ///
  /// Later, you may access this value through the `req.signedCookies` object.
  #[napi]
  pub fn cookie(
    &mut self,
    name: String,
    value: String,
    options: Option<CookieOptions>,
  ) -> Result<()> {
    let mut inner = self.unwrap_inner_or_default();

    let mut option_string = String::new();

    if let Some(options) = &options {
      let mut options_strings = Vec::new();

      if let Some(expires) = &options.expires {
        let expires = httpdate::fmt_http_date(utilities::js_date_to_system_time(expires)?);
        options_strings.push(format!("Expires={expires}"));
      }

      if let Some(max_age) = &options.max_age {
        options_strings.push(format!("Max-Age={max_age}"));
      }

      if let Some(domain) = &options.domain {
        options_strings.push(format!("Domain={domain}"));
      }

      match &options.path {
        Some(path) => {
          options_strings.push(format!("Path={path}"));
        }
        None => {
          options_strings.push("Path=/".to_owned());
        }
      }

      if let Some(secure) = options.secure
        && secure
      {
        options_strings.push("Secure".to_owned());
      }

      if let Some(http_only) = options.http_only
        && http_only
      {
        options_strings.push("HttpOnly".to_owned());
      }

      if !options_strings.is_empty() {
        option_string = options_strings.join("; ")
      }
    }

    let value = match options.and_then(|o| o.encode) {
      Some(js_fn) => js_fn.call(value)?,
      None => urlencoding::encode(&value).to_string(),
    };

    let cookie_string = [format!("{name}={value}"), option_string].join("; ");

    let header_value = HeaderValue::from_str(&cookie_string)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    inner.headers_mut().append(SET_COOKIE, header_value);

    self.set_inner(inner)
  }
}

#[cfg(test)]
mod tests {
  use hyper::header::{GetAll, HeaderValue, SET_COOKIE};

  use crate::response::cookie::CookieOptions;

  use super::Response;

  fn assert_exists(header_values: &GetAll<'_, HeaderValue>, value: &str) {
    assert!(header_values.iter().any(|v| v.to_str().unwrap() == value))
  }

  #[test]
  fn cookie_multiple() {
    let mut res = Response::new();

    let options = CookieOptions {
      path: Some("/".to_owned()),
      secure: Some(true),
      http_only: Some(true),
      ..Default::default()
    };
    res
      .cookie(
        "SID".to_owned(),
        "31d4d96e407aad42".to_owned(),
        Some(options),
      )
      .unwrap();

    let options = CookieOptions {
      path: Some("/".to_owned()),
      domain: Some("example.com".to_owned()),
      ..Default::default()
    };
    res
      .cookie("lang".to_owned(), "en-US".to_owned(), Some(options))
      .unwrap();

    let inner = res.inner().unwrap();
    let header_values = inner.headers().get_all(SET_COOKIE.as_str());

    assert_exists(
      &header_values,
      "SID=31d4d96e407aad42; Path=/; Secure; HttpOnly",
    );
    assert_exists(&header_values, "lang=en-US; Domain=example.com; Path=/");
  }

  #[test]
  fn cookie_default_value_encoding() {
    let mut res = Response::new();

    let options = CookieOptions {
      domain: Some("example.com".to_owned()),
      ..Default::default()
    };
    res
      .cookie(
        "some_cross_domain_cookie".to_owned(),
        "http://mysubdomain.example.com".to_owned(),
        Some(options),
      )
      .unwrap();

    let inner = res.inner().unwrap();
    let header_values = inner.headers().get_all(SET_COOKIE.as_str());
    assert_exists(
      &header_values,
      "some_cross_domain_cookie=http%3A%2F%2Fmysubdomain.example.com; Domain=example.com; Path=/",
    );
  }
}
