use bytes::Bytes;
use hyper::header::COOKIE;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{request::Request, response::Response, utilities};

#[napi(object)]
pub struct JsCookieParserOptions {
  /// If `true`, the cookie value will be percent-decoded
  ///
  /// A cookie whose value is `foo=bar%20baz; HttpOnly` gets parsed into the
  /// name-value pair `"foo", "bar baz"`
  pub percent_decode: Option<bool>,
}

struct CookieParserOptions {
  percent_decode: bool,
}

impl Default for CookieParserOptions {
  fn default() -> Self {
    Self {
      percent_decode: true,
    }
  }
}

impl JsCookieParserOptions {
  fn to_raw_options(&self) -> Result<CookieParserOptions> {
    let mut cookie_parse_options = CookieParserOptions::default();

    if let Some(percent_decode) = self.percent_decode {
      cookie_parse_options.percent_decode = percent_decode
    }

    Ok(cookie_parse_options)
  }
}

/// Create a new cookie parser middleware function using the given secret and
/// options.
///
/// - `secret` a string or array used for signing cookies. This is optional and
/// if not specified, will not parse signed cookies. If a string is provided,
/// this is used as the secret. If an array is provided, an attempt will be
/// made to unsign the cookie with each secret in order.
///
/// - `options` an object that is passed to cookie.parse as the second option. See cookie for more information.
/// decode a function to decode the value of the cookie
/// The middleware will parse the Cookie header on the request and expose the cookie data as the property req.cookies and, if a secret was provided, as the property req.signedCookies. These properties are name value pairs of the cookie name to cookie value.
#[napi]
pub struct CookieParserMiddleware {
  secrets: Vec<Bytes>,
  #[allow(unused)]
  options: CookieParserOptions,
}

#[napi]
impl CookieParserMiddleware {
  #[napi(constructor)]
  pub fn new(
    secret: Option<Either<String, Vec<String>>>,
    options: Option<JsCookieParserOptions>,
  ) -> Result<Self> {
    let secrets = match secret.as_ref() {
      Some(secret) => match secret {
        Either::A(secret) => vec![secret],
        Either::B(secrets) => secrets.iter().collect(),
      },
      None => Vec::with_capacity(0),
    }
    .iter()
    .map(|secret| Bytes::copy_from_slice(secret.as_bytes()))
    .collect::<Vec<_>>();
    Ok(CookieParserMiddleware {
      secrets,
      options: match options {
        Some(options) => options.to_raw_options()?,
        None => CookieParserOptions::default(),
      },
    })
  }

  #[napi]
  pub async fn run(&self, request: &Request, _response: &Response) -> Result<bool> {
    println!("CookieParser Middleware | Called!");

    request.with_inner_mut(|w_req| {
      let Some(cookie_header) = w_req.inner()?.headers().get(COOKIE) else {
        return Ok(true);
      };
      let request_cookies = utilities::extract_cookies(&self.secrets, cookie_header)?;
      w_req.set_encrypted_cookies(request_cookies.encrypted);
      w_req.set_cookies(request_cookies.unencrypted);
      Ok(true)
    })
  }
}
