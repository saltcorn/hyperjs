use headers_core::HeaderValue;
use hyper::header::LINK;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Response, WrappedResponse};

#[napi]
impl Response {
  /// Joins the links provided as properties of the parameter to populate the response’s Link HTTP header field.
  ///
  /// For example, the following call:
  ///
  /// ```javascript
  /// res.links({
  ///   next: 'http://api.example.com/users?page=2',
  ///   last: 'http://api.example.com/users?page=5'
  /// })
  /// ```
  ///
  /// Yields the following results:
  ///
  /// ```text
  /// Link: <http://api.example.com/users?page=2>; rel="next",
  ///       <http://api.example.com/users?page=5>; rel="last"
  /// ```
  #[napi]
  pub fn links(&mut self, links: Object) -> Result<()> {
    self.with_inner(|response| response.links(links))
  }
}

impl WrappedResponse {
  pub fn links(&mut self, links: Object) -> Result<()> {
    let mut all_links = self
      .inner()?
      .headers()
      .get(LINK)
      .and_then(|v| v.to_str().map(|s| vec![s.to_owned()]).ok())
      .unwrap_or_default();

    for key in Object::keys(&links)? {
      let value = links
        .get::<Either<String, Vec<String>>>(&key)?
        .ok_or(Error::new(
          Status::GenericFailure,
          "Expected key to have an associated value.",
        ))?;
      match value {
        Either::A(string_value) => all_links.push(format!(r#"<{string_value}>; rel="{key}""#)),
        Either::B(strings_list) => {
          all_links.push(format!(r#"<{}>; rel="{key}""#, strings_list.join(", ")))
        }
      }
    }

    let link_header_value_str = all_links.join(", ");
    let final_value = HeaderValue::from_str(&link_header_value_str).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!(r#"Invalid HeaderValue "{link_header_value_str}": {e}"#),
      )
    })?;

    self.inner()?.headers_mut().insert(LINK, final_value);

    Ok(())
  }
}
