use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::StatusCode;

use crate::utilities::{empty, full};

pub fn body_from_status_code(status_code: StatusCode) -> BoxBody<Bytes, hyper::Error> {
  match status_code.canonical_reason() {
    Some(reason) => full(reason.as_bytes()),
    None => empty(),
  }
}
