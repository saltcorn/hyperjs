use std::{collections::HashSet, str::FromStr};

use hyper::header::{HeaderName, HeaderValue, VARY};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Response, WrappedResponse};

#[napi]
impl Response {
  /// Adds the field to the `Vary` response header, if it is not there already.
  ///
  /// ``` javascript
  /// res.vary('User-Agent').render('docs')
  /// ```
  #[napi]
  pub fn vary(&mut self, field: String) -> Result<()> {
    self.with_inner(|response| response.vary(field))
  }
}

impl WrappedResponse {
  fn merge_vary_headers(vary_headers: &[&str], field: String) -> Result<String> {
    let supplied_headers = field.split(",").map(|s| s.trim()).collect::<Vec<_>>();

    // validate that each of the provided values is a valid header name
    for header in &supplied_headers {
      if let Err(_e) = HeaderName::from_str(header) {
        return Err(Error::new(
          Status::InvalidArg,
          format!("field argument contains an invalid header name: '{header}'"),
        ));
      }
    }

    // existing, unspecified vary
    if vary_headers.contains(&"*") {
      return Ok("*".to_owned());
    }

    // unspecified vary
    if supplied_headers.contains(&"*") {
      return Ok("*".to_owned());
    }

    let mut headers_set = HashSet::new();

    for header in vary_headers {
      headers_set.insert(*header);
    }

    for header in supplied_headers {
      headers_set.insert(header);
    }

    let merged_headers = headers_set.into_iter().collect::<Vec<_>>().join(", ");

    Ok(merged_headers)
  }

  pub fn vary(&mut self, field: String) -> Result<()> {
    let mut vary_header_values = Vec::new();
    for header_value in self.inner()?.headers().get_all(VARY) {
      let Ok(header_value) = header_value.to_str() else {
        return Err(Error::new(
          Status::GenericFailure,
          "Could not convert VARY header value to string.",
        ));
      };
      vary_header_values.push(header_value);
    }
    let vary_header_value = Self::merge_vary_headers(&vary_header_values, field)?;
    let vary_header_value = HeaderValue::from_str(&vary_header_value)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    // set new header
    self.inner()?.headers_mut().insert(VARY, vary_header_value);

    Ok(())
  }
}
