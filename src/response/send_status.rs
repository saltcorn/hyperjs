use hyper::StatusCode as LibStatusCode;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::Response;
use crate::response::status_code::StatusCode;

#[napi]
impl Response {
  /// Sets the response HTTP status code to statusCode and sends the registered
  /// status message as the text response body. If an unknown status code is
  /// specified,`res.statusCode` will throw an error
  ///
  /// ```javascript
  /// res.sendStatus(404)
  /// ```
  #[napi]
  pub fn send_status(&mut self, body: Either<u16, &StatusCode>, env: Env) -> Result<()> {
    let status_code = match body {
      Either::A(value) => {
        LibStatusCode::from_u16(value).map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?
      }
      Either::B(value) => value.inner().to_owned(),
    };

    *self.inner()?.status_mut() = status_code;

    let body = match status_code.canonical_reason() {
      Some(reason) => Either6::A(reason.to_owned()),
      None => Either6::E(Null),
    };

    self.send(body, env)
  }
}
