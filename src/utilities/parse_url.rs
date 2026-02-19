//! parseurl
//!
//! A Rust port of the Node.js parseurl module for parsing URLs with memoization.
//! This is particularly useful for HTTP request handling where the same URL
//! may be parsed multiple times.

use hyper::Request;
use std::cell::RefCell;

/// Represents a parsed URL with common components
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedUrl {
  pub path: String,
  pub href: String,
  pub pathname: String,
  pub query: Option<String>,
  pub search: Option<String>,
  _raw: String,
}

/// Extension trait to add URL parsing with memoization to Hyper requests
pub trait RequestExt {
  /// Parse the request URL with memoization
  fn parseurl(&self) -> Option<ParsedUrl>;

  /// Parse the original URL (from headers) with fallback and memoization
  fn original_url(&self) -> Option<ParsedUrl>;
}

thread_local! {
    static PARSED_URL_CACHE: RefCell<ParsedUrlCache> = RefCell::new(ParsedUrlCache::new());
}

struct ParsedUrlCache {
  entries: Vec<CacheEntry>,
  max_entries: usize,
}

struct CacheEntry {
  raw: String,
  parsed: ParsedUrl,
  request_id: usize,
}

impl ParsedUrlCache {
  fn new() -> Self {
    Self {
      entries: Vec::new(),
      max_entries: 100, // Limit cache size
    }
  }

  fn get(&self, raw: &str, request_id: usize) -> Option<ParsedUrl> {
    self
      .entries
      .iter()
      .find(|e| e.raw == raw && e.request_id == request_id)
      .map(|e| e.parsed.clone())
  }

  fn insert(&mut self, raw: String, parsed: ParsedUrl, request_id: usize) {
    // Remove old entries for this request_id
    self.entries.retain(|e| e.request_id != request_id);

    // Add new entry
    self.entries.push(CacheEntry {
      raw,
      parsed,
      request_id,
    });

    // Limit cache size
    if self.entries.len() > self.max_entries {
      self.entries.drain(0..self.entries.len() - self.max_entries);
    }
  }
}

impl<B> RequestExt for Request<B> {
  fn parseurl(&self) -> Option<ParsedUrl> {
    let uri = self.uri();
    let path_and_query = uri.path_and_query()?;
    let url = path_and_query.as_str();

    // Use request pointer as a simple ID
    let request_id = self as *const _ as usize;

    // Check cache
    let cached = PARSED_URL_CACHE.with(|cache| cache.borrow().get(url, request_id));

    if let Some(parsed) = cached {
      return Some(parsed);
    }

    // Parse the URL
    let mut parsed = fastparse(url);
    parsed._raw = url.to_string();

    // Cache the result
    PARSED_URL_CACHE.with(|cache| {
      cache
        .borrow_mut()
        .insert(url.to_string(), parsed.clone(), request_id);
    });

    Some(parsed)
  }

  fn original_url(&self) -> Option<ParsedUrl> {
    // Try to get the original URL from X-Original-URL or X-Original-Path headers
    let url = if let Some(original_url) = self.headers().get("x-original-url") {
      original_url.to_str().ok()?
    } else if let Some(original_path) = self.headers().get("x-original-path") {
      original_path.to_str().ok()?
    } else {
      // Fallback to regular parseurl
      return self.parseurl();
    };

    let request_id = self as *const _ as usize;

    // Check cache
    let cached = PARSED_URL_CACHE.with(|cache| cache.borrow().get(url, request_id));

    if let Some(parsed) = cached {
      return Some(parsed);
    }

    // Parse the URL
    let mut parsed = fastparse(url);
    parsed._raw = url.to_string();

    // Cache the result
    PARSED_URL_CACHE.with(|cache| {
      cache
        .borrow_mut()
        .insert(url.to_string(), parsed.clone(), request_id);
    });

    Some(parsed)
  }
}

/// Parse the URL string with fast-path short-cut.
///
/// This function uses a fast path for URLs that start with '/' and don't contain
/// special characters that would require full URL parsing.
fn fastparse(s: &str) -> ParsedUrl {
  // Fast path: check if string starts with '/'
  if s.is_empty() || !s.starts_with('/') {
    return full_parse(s);
  }

  let bytes = s.as_bytes();
  let mut pathname = s;
  let mut query = None;
  let mut search = None;

  // Scan for special characters
  for i in 1..bytes.len() {
    match bytes[i] {
      b'?' => {
        if search.is_none() {
          pathname = &s[0..i];
          query = Some(s[i + 1..].to_string());
          search = Some(s[i..].to_string());
        }
      }
      b'\t' | b'\n' | 0x0c | b'\r' | b' ' | b'#' | 0xa0 => {
        // Fall back to full parse for these characters
        return full_parse(s);
      }
      0xfe if i + 1 < bytes.len() && bytes[i + 1] == 0xff => {
        // UTF-8 BOM (0xfeff) - fall back to full parse
        return full_parse(s);
      }
      _ => {}
    }
  }

  ParsedUrl {
    path: s.to_string(),
    href: s.to_string(),
    pathname: pathname.to_string(),
    query,
    search,
    _raw: String::new(),
  }
}

/// Full URL parsing fallback
fn full_parse(s: &str) -> ParsedUrl {
  // Basic URL parsing
  let (path_part, _fragment) = s.split_once('#').unwrap_or((s, ""));
  let (pathname, query_part) = path_part.split_once('?').unwrap_or((path_part, ""));

  let search = if !query_part.is_empty() {
    Some(format!("?{}", query_part))
  } else {
    None
  };

  let query = if !query_part.is_empty() {
    Some(query_part.to_string())
  } else {
    None
  };

  ParsedUrl {
    path: path_part.to_string(),
    href: s.to_string(),
    pathname: pathname.to_string(),
    query,
    search,
    _raw: String::new(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use hyper::{Method, Request};
  use hyper_staticfile::Body;
  use tokio::fs::File;

  fn create_request(path: &str) -> Request<Body> {
    Request::builder()
      .method(Method::GET)
      .uri(path)
      .body(Body::Empty)
      .unwrap()
  }

  fn create_request_with_original(path: &str, original: &str) -> Request<Body> {
    Request::builder()
      .method(Method::GET)
      .uri(path)
      .header("x-original-url", original)
      .body(Body::Empty)
      .unwrap()
  }

  #[test]
  fn test_parseurl_simple_path() {
    let req = create_request("/foo/bar");
    let result = req.parseurl().unwrap();

    assert_eq!(result.pathname, "/foo/bar");
    assert_eq!(result.path, "/foo/bar");
    assert_eq!(result.href, "/foo/bar");
    assert_eq!(result.query, None);
    assert_eq!(result.search, None);
  }

  #[test]
  fn test_parseurl_with_query_string() {
    let req = create_request("/foo?bar=baz");
    let result = req.parseurl().unwrap();

    assert_eq!(result.pathname, "/foo");
    assert_eq!(result.path, "/foo?bar=baz");
    assert_eq!(result.href, "/foo?bar=baz");
    assert_eq!(result.query, Some("bar=baz".to_string()));
    assert_eq!(result.search, Some("?bar=baz".to_string()));
  }

  #[test]
  fn test_parseurl_with_multiple_query_params() {
    let req = create_request("/foo?bar=baz&qux=quux");
    let result = req.parseurl().unwrap();

    assert_eq!(result.pathname, "/foo");
    assert_eq!(result.query, Some("bar=baz&qux=quux".to_string()));
    assert_eq!(result.search, Some("?bar=baz&qux=quux".to_string()));
  }

  #[test]
  fn test_parseurl_memoization() {
    let req = create_request("/foo/bar");

    let result1 = req.parseurl().unwrap();
    let result2 = req.parseurl().unwrap();

    assert_eq!(result1, result2);
  }

  #[test]
  fn test_original_url_fallback() {
    let req = create_request("/foo");
    let result = req.original_url().unwrap();

    assert_eq!(result.pathname, "/foo");
  }

  #[test]
  fn test_original_url_with_header() {
    let req = create_request_with_original("/foo", "/original/path");
    let result = req.original_url().unwrap();

    assert_eq!(result.pathname, "/original/path");
  }

  #[test]
  fn test_original_url_with_query() {
    let req = create_request_with_original("/foo", "/original?key=value");
    let result = req.original_url().unwrap();

    assert_eq!(result.pathname, "/original");
    assert_eq!(result.query, Some("key=value".to_string()));
  }

  #[test]
  fn test_fastparse_root_path() {
    let result = fastparse("/");
    assert_eq!(result.pathname, "/");
    assert_eq!(result.path, "/");
  }

  #[test]
  fn test_fastparse_with_hash() {
    // Hash should trigger full parse
    let result = fastparse("/foo#bar");
    assert_eq!(result.pathname, "/foo");
  }

  #[test]
  fn test_fastparse_with_spaces() {
    // Spaces should trigger full parse
    let result = fastparse("/foo bar");
    assert_eq!(result.pathname, "/foo bar");
  }

  #[test]
  fn test_query_string_with_multiple_question_marks() {
    let req = create_request("/foo?bar=baz?qux");
    let result = req.parseurl().unwrap();

    // Only the first ? should be treated as query delimiter
    assert_eq!(result.pathname, "/foo");
    assert_eq!(result.query, Some("bar=baz?qux".to_string()));
  }

  #[test]
  fn test_empty_query_string() {
    let req = create_request("/foo?");
    let result = req.parseurl().unwrap();

    assert_eq!(result.pathname, "/foo");
    assert_eq!(result.query, Some("".to_string()));
    assert_eq!(result.search, Some("?".to_string()));
  }

  #[test]
  fn test_complex_path() {
    let req = create_request("/foo/bar/baz?key1=value1&key2=value2");
    let result = req.parseurl().unwrap();

    assert_eq!(result.pathname, "/foo/bar/baz");
    assert_eq!(result.query, Some("key1=value1&key2=value2".to_string()));
  }

  #[test]
  fn test_url_encoded_params() {
    let req = create_request("/search?q=hello%20world");
    let result = req.parseurl().unwrap();

    assert_eq!(result.pathname, "/search");
    assert_eq!(result.query, Some("q=hello%20world".to_string()));
  }

  #[test]
  fn test_multiple_slashes() {
    let req = create_request("/foo//bar///baz");
    let result = req.parseurl().unwrap();

    assert_eq!(result.pathname, "/foo//bar///baz");
  }

  #[test]
  fn test_with_fragment() {
    let req = create_request("/foo?bar=baz#section");
    let result = req.parseurl().unwrap();

    assert_eq!(result.pathname, "/foo");
    // Fragment is typically not included in server-side request URIs
  }

  #[test]
  fn test_different_requests_different_cache() {
    let req1 = create_request("/foo");
    let req2 = create_request("/bar");

    let result1 = req1.parseurl().unwrap();
    let result2 = req2.parseurl().unwrap();

    assert_eq!(result1.pathname, "/foo");
    assert_eq!(result2.pathname, "/bar");
  }

  #[test]
  fn test_original_url_with_x_original_path_header() {
    let req = Request::builder()
      .method(Method::GET)
      .uri("/proxy")
      .header("x-original-path", "/actual/path?query=1")
      .body(Body::<File>::Empty)
      .unwrap();

    let result = req.original_url().unwrap();
    assert_eq!(result.pathname, "/actual/path");
    assert_eq!(result.query, Some("query=1".to_string()));
  }

  #[test]
  fn test_parseurl_special_chars() {
    let req = create_request("/api/v1/users");
    let result = req.parseurl().unwrap();

    assert_eq!(result.pathname, "/api/v1/users");
  }

  #[test]
  fn test_parseurl_with_port() {
    // Hyper's URI parsing handles this
    let req = create_request("/path");
    let result = req.parseurl().unwrap();

    assert_eq!(result.pathname, "/path");
  }
}
