use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

use askama_escape::escape_html;
use hyper::{StatusCode, header::LOCATION};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Response, WrappedResponse};
use crate::request::Request;

#[napi]
impl Response {
  /// Redirects to the URL derived from the specified `path`, with specified
  /// `status`, a positive integer that corresponds to an
  /// [HTTP status code](https://www.rfc-editor.org/rfc/rfc9110.html#name-status-codes).
  /// If not specified, `status` defaults to `302 "Found"`.
  ///
  /// ```javascript
  /// res.redirect('/foo/bar')
  /// res.redirect('http://example.com')
  /// res.redirect(301, 'http://example.com')
  /// res.redirect('../login')
  /// ```
  ///
  /// Redirects can be a fully-qualified URL for redirecting to a different site:
  ///
  /// ```javascript
  /// res.redirect('http://google.com')
  /// ```
  ///
  /// Redirects can be relative to the root of the host name. For example, if
  /// the application is on `http://example.com/admin/post/new`, the following
  /// would redirect to the URL `http://example.com/admin`:
  ///
  /// ```javascript
  /// res.redirect('/admin')
  /// ```
  ///
  /// Redirects can be relative to the current URL. For example, from
  /// `http://example.com/blog/admin/` (notice the trailing slash), the
  /// following would redirect to the URL
  /// `http://example.com/blog/admin/post/new`.
  ///
  /// ```javascript
  /// res.redirect('post/new')
  /// ```
  ///
  /// Redirecting to `post/new` from `http://example.com/blog/admin` (no
  /// trailing slash), will redirect to `http://example.com/blog/post/new`.
  ///
  /// If you found the above behavior confusing, think of path segments as
  /// directories (with trailing slashes) and files, it will start to make
  /// sense.
  ///
  /// Path-relative redirects are also possible. If you were on
  /// `http://example.com/admin/post/new`, the following would redirect to
  /// `http://example.com/admin/post`:
  ///
  /// ```javascript
  /// res.redirect('..')
  /// ```
  ///
  /// See also [Security best practices: Prevent open redirect vulnerabilities](http://expressjs.com/en/advanced/best-practice-security.html#prevent-open-redirects).

  #[napi]
  pub fn redirect(
    &mut self,
    status: Either<u16, String>,
    address: Option<String>,
    env: Env,
  ) -> Result<()> {
    let (status, address) = match (status, address) {
      (Either::A(status), Some(address)) => (status, address),
      (Either::B(address), None) => (302, address),
      (Either::B(_), Some(_)) => {
        return Err(Error::new(Status::InvalidArg, "Status must be an integer."));
      }
      _ => return Err(Error::new(Status::InvalidArg, "Provide a url argument.")),
    };

    // set location header
    self.location(address)?;

    let Either::A(address) = self.get(LOCATION.to_string())? else {
      return Err(Error::new(
        Status::GenericFailure,
        "Expected location header value to be a string.",
      ));
    };

    // set status
    self.status(Either::A(status))?;

    let status = StatusCode::from_u16(status)
      .map_err(|_e| Error::new(Status::InvalidArg, format!("Invalid status code: {status}")))?;

    let body = Arc::new(Mutex::new(Some(String::new())));

    let mut format_fns: HashMap<String, Function<FnArgs<(Request, Response)>, ()>> = HashMap::new();

    // Support text/{plain,html} by default
    let body_clone = body.clone();
    let address_clone = address.clone();
    let text_fn =
      env.create_function_from_closure("textFn", move |_ctx: FunctionCallContext<'_>| {
        match body_clone.lock() {
          Ok(mut body) => {
            let _ = body.insert(format!(
              "{}. Redirecting to {address_clone}",
              status.canonical_reason().unwrap_or_default()
            ));
            Ok(())
          }
          Err(_) => Err(Error::new(
            Status::GenericFailure,
            "Could not obtain lock on body variable.",
          )),
        }
      })?;

    format_fns.insert("text".to_owned(), text_fn);

    let mut escaped_address = String::new();
    escape_html(&mut escaped_address, &address)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
    let body_clone = body.clone();
    let html_fn =
      env.create_function_from_closure("htmlFn", move |_ctx: FunctionCallContext<'_>| {
        match body_clone.lock() {
          Ok(mut body) => {
            let _ = body.insert(format!(
              r#"<!DOCTYPE html><head><title>{}</title></head><body><p>{}. Redirecting to {escaped_address}</p></body>"#,
              status.canonical_reason().unwrap_or_default(),
              status.canonical_reason().unwrap_or_default()
            ));
            Ok(())
          }
          Err(_) => Err(Error::new(
            Status::GenericFailure,
            "Could not obtain lock on body variable.",
          )),
        }
      })?;

    format_fns.insert("html".to_owned(), html_fn);

    let body_clone = body.clone();
    let default_fn =
      env.create_function_from_closure("defaultFn", move |_ctx: FunctionCallContext<'_>| {
        match body_clone.lock() {
          Ok(mut body) => {
            body.take();
            Ok(())
          }
          Err(_) => Err(Error::new(
            Status::GenericFailure,
            "Could not obtain lock on body variable.",
          )),
        }
      })?;

    format_fns.insert("default".to_owned(), default_fn);

    let req = self.req();
    let req_method = req.method()?.to_lowercase();
    WrappedResponse::formut(format_fns, req, self.clone())?;

    let body = body
      .lock()
      .map(|b| b.to_owned().map(Either3::A))
      .map_err(|_| {
        Error::new(
          Status::GenericFailure,
          "Could not obtain lock on body variable.",
        )
      })?;

    match req_method.to_lowercase().as_str() {
      "head" => self.end(None),
      _ => self.end(body),
    }
  }
}
