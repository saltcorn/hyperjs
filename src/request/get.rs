use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Request, WrappedRequest};

#[napi]
impl Request {
  /// Returns the specified HTTP request header field (case-insensitive match).
  /// The `Referrer` and `Referer` fields are interchangeable.
  ///
  /// ```javascript
  /// req.get('Content-Type')
  /// // => "text/plain"
  ///
  /// req.get('content-type')
  /// // => "text/plain"
  ///
  /// req.get('Something')
  /// // => undefined
  /// ```
  ///
  /// Aliased as `req.header(field)`.
  #[napi]
  pub fn get(&self, field: String) -> Result<Either<String, Buffer>> {
    self.with_inner(|request| request.get(field))
  }

  #[napi]
  pub fn header(&self, field: String) -> Result<Either<String, Buffer>> {
    self.with_inner(|request| request.get(field))
  }
}

impl WrappedRequest {
  pub fn get(&self, field: String) -> Result<Either<String, Buffer>> {
    let header_values = self
      .inner()?
      .headers()
      .get_all(&field)
      .iter()
      .map(|value| match value.to_str() {
        Ok(value) => Either::A(value),
        Err(_) => Either::B(value.as_bytes()),
      })
      .collect::<Vec<_>>();

    match header_values.iter().any(|v| match v {
      Either::A(_) => false,
      Either::B(_) => true,
    }) {
      true => {
        let byte_values = header_values
          .iter()
          .map(|v| match v {
            Either::A(a) => a.as_bytes(),
            Either::B(b) => b,
          })
          .collect::<Vec<_>>()
          .join(&b", "[..]);
        Ok(Either::B(byte_values.into()))
      }
      false => {
        let str_values = header_values
          .iter()
          .filter_map(|v| match v {
            Either::A(a) => Some(*a),
            Either::B(_) => None,
          })
          .collect::<Vec<_>>()
          .join(", ");
        Ok(Either::A(str_values))
      }
    }
  }
}
