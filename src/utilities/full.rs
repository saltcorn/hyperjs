use bytes::Bytes;
use http_body_util::Full;

use crate::response::CrateBody;

// Utility function to make Full bodies.
pub fn full<T: Into<Bytes>>(chunk: T) -> CrateBody {
  CrateBody::Full(Full::new(chunk.into()))
}
