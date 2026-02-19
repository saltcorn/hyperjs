use hyper::StatusCode;

use crate::{response::CrateBody, utilities::full};

pub fn body_from_status_code(status_code: StatusCode) -> CrateBody {
  match status_code.canonical_reason() {
    Some(reason) => full(reason.as_bytes()),
    None => CrateBody::Empty,
  }
}
