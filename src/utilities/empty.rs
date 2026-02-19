use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use http_body_util::{BodyExt, Empty};
use hyper::Error as LibError;

// Utility functions to make Empty bodies.
pub fn empty() -> BoxBody<Bytes, LibError> {
  Empty::<Bytes>::new()
    .map_err(|never| match never {})
    .boxed()
}
