use std::{collections::HashMap, str::FromStr};

use hyper::header::{HeaderName, HeaderValue};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use serde_json::Value as JsonValue;

use crate::utilities;

use super::{Response, WrappedResponse};

#[napi]
impl Response {
  /// Sets the response’s HTTP header field to value. To set multiple fields at
  /// once, pass an object as the parameter.
  ///
  /// ```javascript
  /// res.set('Content-Type', 'text/plain')
  ///
  /// res.set({
  ///   'Content-Type': 'text/plain',
  ///   'Content-Length': '123',
  ///   ETag: '12345',
  ///   Link: ['<http://localhost/>', '<http://localhost:3000/>']
  /// })
  /// ```
  ///
  /// Aliased as res.header(field [, value]).
  #[napi]
  pub fn set(
    &mut self,
    field: Either<String, Object>,
    value: Option<String>,
    env: Env,
  ) -> Result<Self> {
    self.with_inner(|response| response.set(field, value, env))?;
    Ok(self.to_owned())
  }
}

impl Response {
  pub fn set_string(&mut self, field: String, value: String) -> Result<()> {
    self.with_inner(|response| response.set_string(field, value))
  }
}

impl WrappedResponse {
  pub fn set_string(&mut self, field: String, value: String) -> Result<()> {
    let headers_map = self.inner()?.headers_mut();
    let header_name = HeaderName::from_str(&field)
      .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid header name. {e}")))?;
    let header_value = HeaderValue::from_str(&value)
      .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid header value. {e}")))?;
    headers_map.insert(header_name, header_value);
    Ok(())
  }

  pub fn set_object(&mut self, obj: Object, env: Env) -> Result<()> {
    let headers_object: JsonValue = env.from_js_value(obj)?;
    let headers = match headers_object {
      JsonValue::Object(headers_map) => {
        let mut headers = HashMap::new();
        for (field, value) in headers_map {
          headers.insert(field, utilities::json_value_as_string(value)?);
        }
        headers
      }
      _ => {
        return Err(Error::new(
          Status::InvalidArg,
          "Field should be an object or string.",
        ));
      }
    };

    let headers_map = self.inner()?.headers_mut();

    for (field, value) in headers {
      let header_name = HeaderName::from_str(&field)
        .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid header name. {e}")))?;
      match value {
        Either::A(value) => {
          let header_value = HeaderValue::from_str(&value)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid header value. {e}")))?;
          headers_map.insert(header_name, header_value);
        }
        Either::B(values) => {
          for value in values {
            let header_value = HeaderValue::from_str(&value)
              .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid header value. {e}")))?;
            headers_map.append(&header_name, header_value);
          }
        }
      }
    }

    Ok(())
  }

  pub fn set(
    &mut self,
    field: Either<String, Object>,
    value: Option<String>,
    env: Env,
  ) -> Result<()> {
    match field {
      Either::A(field) => match value {
        Some(value) => self.set_string(field, value),
        None => Err(Error::new(
          Status::InvalidArg,
          "Field's value not provided.",
        )),
      },
      Either::B(headers_object) => self.set_object(headers_object, env),
    }
  }
}
