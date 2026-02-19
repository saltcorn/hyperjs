use headers_core::HeaderValue;
use hyper::header::LOCATION;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Response, WrappedResponse};
use crate::utilities;

#[napi]
impl Response {
  /// Sets the response Location HTTP header to the specified path parameter.
  ///
  /// ```javascript
  /// res.location('/foo/bar')
  /// res.location('http://example.com')
  /// ```
  ///
  /// > After encoding the URL, if not encoded already, the specified URL is
  /// > passed to the browser in the Location header, without any validation.
  /// >
  /// > Browsers take the responsibility of deriving the intended URL from the
  /// > current URL or the referring URL, and the URL specified in the Location
  /// > header; and redirect the user accordingly.
  ///
  /// The given `url` can also be "back", which redirects to the _Referrer_ or
  /// _Referer_ headers or "/".
  #[napi]
  pub fn location(&mut self, url: String) -> Result<Self> {
    self.with_inner(|response| response.location(url))?;
    Ok(self.clone())
  }
}

impl WrappedResponse {
  pub fn location(&mut self, url: String) -> Result<()> {
    let url = utilities::encode_url(&url);
    let url = HeaderValue::from_str(&url).map_err(|e| {
      Error::new(
        Status::InvalidArg,
        format!(r#"Invalid HeaderValue "{url}": {e}"#),
      )
    })?;

    self.inner()?.headers_mut().insert(LOCATION, url);

    Ok(())
  }
}
