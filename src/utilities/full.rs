use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Full};
use hyper::Error as LibError;

// Utility function to make Full bodies.
pub fn full<T: Into<Bytes>>(chunk: T) -> BoxBody<Bytes, LibError> {
  Full::new(chunk.into())
    .map_err(|never| match never {})
    .boxed()
}
