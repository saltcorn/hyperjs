use std::str::FromStr;

use hyper::header::{HeaderName, HeaderValue};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::Response;

#[napi]
impl Response {
  /// Appends the specified value to the HTTP response header field. If the header is not already set, it creates the header
  /// with the specified value. The value parameter can be a string or an array.
  /// > **&#10155; Note**
  /// >
  /// > calling `res.set()` after `res.append()` will reset the previously-set header value.
  ///
  /// ```javascript
  /// res.append('Link', ['<http://localhost/>', '<http://localhost:3000/>'])
  /// res.append('Set-Cookie', 'foo=bar; Path=/; HttpOnly')
  /// res.append('Warning', '199 Miscellaneous warning')
  /// ```
  #[napi]
  pub fn append(&mut self, field: String, value: Either<Vec<String>, String>) -> Result<()> {
    let headers_map = self.inner()?.headers_mut();
    let header_name =
      HeaderName::from_str(&field).map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?;
    match value {
      Either::A(values) => {
        for value in values.iter() {
          let header_value = HeaderValue::from_str(value)
            .map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?;
          headers_map.append(&header_name, header_value);
        }
      }
      Either::B(value) => {
        let header_value = HeaderValue::from_str(&value)
          .map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?;
        headers_map.append(header_name, header_value);
      }
    }
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use napi::Either;

  use super::Response;

  #[test]
  fn append() {
    let mut res = Response::new();
    res
      .append(
        "Link".to_owned(),
        Either::A(vec![
          "<http://localhost/>".to_owned(),
          "<http://localhost:3000/>".to_owned(),
        ]),
      )
      .unwrap();
    res
      .append(
        "Set-Cookie".to_owned(),
        Either::B("foo=bar; Path=/; HttpOnly".to_owned()),
      )
      .unwrap();
    res
      .append(
        "Warning".to_owned(),
        Either::B("199 Miscellaneous warning".to_owned()),
      )
      .unwrap();
    let inner = res.inner().unwrap();
    let link_values = inner.headers().get_all("Link");
    assert!(
      link_values
        .iter()
        .any(|v| v.to_str().unwrap() == "<http://localhost/>")
    );
    assert!(
      link_values
        .iter()
        .any(|v| v.to_str().unwrap() == "<http://localhost:3000/>")
    );
  }
}
