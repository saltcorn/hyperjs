use hyper::http::header::{GetAll, HeaderValue};

pub fn assert_header_exists(header_values: &GetAll<'_, HeaderValue>, value: &str) {
  assert!(header_values.iter().any(|v| v.to_str().unwrap() == value))
}
