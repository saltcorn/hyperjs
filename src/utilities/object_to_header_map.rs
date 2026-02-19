use std::str::FromStr;

use headers_core::{HeaderName, HeaderValue};
use hyper::HeaderMap;
use napi::bindgen_prelude::*;

pub fn object_to_header_map(obj: &Object) -> Result<HeaderMap> {
  let mut headers = HeaderMap::new();
  for key in Object::keys(obj)? {
    let Ok(header_name) = HeaderName::from_str(&key) else {
      return Err(Error::new(
        Status::InvalidArg,
        format!("Invalid header name '{key}'"),
      ));
    };
    let header_value = {
      if let Ok(Some(value)) = obj.get::<String>(&key) {
        HeaderValue::from_str(&value).map_err(|_| {
          Error::new(
            Status::InvalidArg,
            format!("Invalid header value '{value}'"),
          )
        })
      } else if let Ok(Some(value)) = obj.get::<Uint8Array>(&key) {
        let value: &[u8] = value.as_ref();
        HeaderValue::from_bytes(value).map_err(|_| {
          Error::new(
            Status::InvalidArg,
            format!("Invalid header value '{value:?}'"),
          )
        })
      } else if let Ok(Some(value)) = obj.get::<Buffer>(&key) {
        let value: &[u8] = value.as_ref();
        HeaderValue::from_bytes(value).map_err(|_| {
          Error::new(
            Status::InvalidArg,
            format!("Invalid header value '{value:?}'"),
          )
        })
      } else if let Ok(Some(value)) = obj.get::<Uint8ArraySlice>(&key) {
        let value: &[u8] = value.as_ref();
        HeaderValue::from_bytes(value).map_err(|_| {
          Error::new(
            Status::InvalidArg,
            format!("Invalid header value '{value:?}'"),
          )
        })
      } else {
        Err(Error::new(
          Status::InvalidArg,
          "Expected header value to be a string or byte array.",
        ))
      }
    }?;
    headers.insert(header_name, header_value);
  }

  Ok(headers)
}
