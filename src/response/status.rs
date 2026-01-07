use hyper::StatusCode as LibStatusCode;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Response, WrappedResponse};
use crate::response::status_code::StatusCode;

#[napi]
impl Response {
  /// Sets the HTTP status for the response.
  ///
  /// ```javascript
  /// res.status(403).end()
  /// res.status(400).send('Bad Request')
  /// res.status(404).sendFile('/absolute/path/to/404.png')
  /// ```
  #[napi]
  pub fn status(&mut self, body: Either<u16, &StatusCode>) -> Result<Response> {
    self.with_inner(|response| response.status(body))?;
    Ok(self.clone())
  }
}

impl WrappedResponse {
  pub fn status(&mut self, body: Either<u16, &StatusCode>) -> Result<()> {
    let status_code = match body {
      Either::A(value) => {
        LibStatusCode::from_u16(value).map_err(|e| Error::new(Status::InvalidArg, e.to_string()))?
      }
      Either::B(value) => value.inner().to_owned(),
    };

    *self.inner()?.status_mut() = status_code;

    Ok(())
  }
}
