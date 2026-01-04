use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty, Full};
use hyper::Error as LibError;

// Utility functions to make Empty and Full bodies.
pub fn empty() -> BoxBody<Bytes, LibError> {
  Empty::<Bytes>::new()
    .map_err(|never| match never {})
    .boxed()
}

pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, LibError> {
  Full::new(chunk.into())
    .map_err(|never| match never {})
    .boxed()
}

use napi::JsDate;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub fn js_date_to_system_time(js_date: &JsDate) -> napi::Result<SystemTime> {
  let millis = js_date.value_of()?;

  if millis >= 0.0 {
    Ok(UNIX_EPOCH + Duration::from_millis(millis as u64))
  } else {
    // For dates before 1970, subtract the duration from the epoch
    Ok(UNIX_EPOCH - Duration::from_millis(millis.abs() as u64))
  }
}
