use hyper::header::CONTENT_TYPE;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::Response;
use crate::utilities;

#[napi]
impl Response {
  /// Sends a JSON response. This method sends a response (with the correct
  /// content-type) that is the parameter converted to a JSON string using
  /// JSON.stringify().
  ///
  /// The parameter can be any JSON type, including object, array, string,
  /// Boolean, number, or null, and you can also use it to convert other values
  /// to JSON.
  ///
  /// ```javascript
  /// res.json(null)
  /// res.json({ user: 'tobi' })
  /// res.status(500).json({ error: 'message' })
  /// ```
  #[napi]
  pub fn json(&mut self, body: Either5<String, i64, bool, Object, Null>, env: Env) -> Result<()> {
    // set `Content-Type` to application/json
    if self.inner()?.headers().get(CONTENT_TYPE).is_none() {
      self.typ("json".to_owned())?
    }

    let body = match body {
      Either5::A(value) => serde_json::to_string(&value)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?,
      Either5::B(value) => serde_json::to_string(&value)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?,
      Either5::C(value) => serde_json::to_string(&value)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?,
      Either5::D(value) => utilities::serialize_napi_object(env, value)?,
      Either5::E(_) => serde_json::to_string(&Option::<String>::None)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?,
    };

    self.send(Either6::A(body), env)
  }
}
