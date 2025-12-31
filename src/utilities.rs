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
