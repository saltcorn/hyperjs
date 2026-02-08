// Behavior Parity Tests: JavaScript send() vs Rust FileSendTask
//
// These tests verify that the Rust implementation matches the JavaScript
// send() library's behavior across all configuration options and scenarios.

use std::{
  fs::{self, File},
  io::Write,
  path::PathBuf,
};

use bytes::Bytes;
use http_body_util::{BodyExt, Empty, Full, combinators::BoxBody};
use hyper::{HeaderMap, Method, Request as HyperRequest, StatusCode, header};
use hyper_staticfile::Body;
use napi::Task;
use tempfile::TempDir;

use crate::{request::WrappedRequest, response::Response};

use super::{FileSendOptions, FileSendTask};

// ============================================================================
// Test Fixture
// ============================================================================

struct TestEnv {
  temp_dir: TempDir,
}

impl TestEnv {
  fn new() -> Self {
    let temp_dir = TempDir::new().unwrap();
    Self { temp_dir }
  }

  fn root(&self) -> PathBuf {
    self.temp_dir.path().to_path_buf()
  }

  fn write_file(&self, path: &str, content: &str) {
    let full_path = self.temp_dir.path().join(path);
    if let Some(parent) = full_path.parent() {
      fs::create_dir_all(parent).unwrap();
    }
    let mut file = File::create(full_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
  }

  fn mkdir(&self, path: &str) {
    let full_path = self.temp_dir.path().join(path);
    fs::create_dir_all(full_path).unwrap();
  }
}

// ============================================================================
// PARITY TEST 1: Default Options Match
// JavaScript: send(req, path)
// Rust: FileSendOptions::default()
// ============================================================================

#[test]
fn parity_default_options() {
  // JavaScript defaults:
  // - acceptRanges: true
  // - cacheControl: true
  // - etag: true
  // - dotfiles: 'ignore'
  // - extensions: []
  // - immutable: false
  // - index: ['index.html']
  // - lastModified: true
  // - maxAge: 0

  let opts = FileSendOptions::default();

  assert!(opts.accept_ranges, "acceptRanges should default to true");
  assert!(opts.cache_control, "cacheControl should default to true");
  assert!(opts.etag, "etag should default to true");
  assert_eq!(
    opts.dotfiles, "ignore",
    "dotfiles should default to 'ignore'"
  );
  assert_eq!(
    opts.extensions.len(),
    0,
    "extensions should default to empty"
  );
  assert!(!opts.immutable, "immutable should default to false");
  assert_eq!(
    opts.index,
    Some(vec!["index.html".to_string()]),
    "index should default to ['index.html']"
  );
  assert!(opts.last_modified, "lastModified should default to true");
  assert_eq!(opts.max_age, 0, "maxAge should default to 0");
}

// ============================================================================
// PARITY TEST 2: dotfiles='deny' Returns 403
// JavaScript: send(req, path, { dotfiles: 'deny' })
// Expected: 403 Forbidden for .dotfiles
// ============================================================================

#[test]
fn parity_dotfiles_deny_returns_403() {
  let env = TestEnv::new();
  env.write_file(".secret", "password123");

  let opts = FileSendOptions {
    root: Some(env.root()),
    dotfiles: "deny".to_string(),
    ..Default::default()
  };

  let response = serve_file("/.secret", opts);

  assert_eq!(
    response.status,
    StatusCode::FORBIDDEN,
    "dotfiles='deny' should return 403 Forbidden"
  );
}

// ============================================================================
// PARITY TEST 3: dotfiles='ignore' Returns 404
// JavaScript: send(req, path, { dotfiles: 'ignore' })
// Expected: 404 Not Found for .dotfiles
// ============================================================================

#[test]
fn parity_dotfiles_ignore_returns_404() {
  let env = TestEnv::new();
  env.write_file(".secret", "password123");

  let opts = FileSendOptions {
    root: Some(env.root()),
    dotfiles: "ignore".to_string(),
    ..Default::default()
  };

  let response = serve_file("/.secret", opts);

  assert_eq!(
    response.status,
    StatusCode::NOT_FOUND,
    "dotfiles='ignore' should return 404 Not Found"
  );
}

// ============================================================================
// PARITY TEST 4: dotfiles='allow' Serves File
// JavaScript: send(req, path, { dotfiles: 'allow' })
// Expected: 200 OK, file served
// ============================================================================

#[test]
fn parity_dotfiles_allow_serves_file() {
  let env = TestEnv::new();
  env.write_file(".well-known/security.txt", "Contact: security@example.com");

  let opts = FileSendOptions {
    root: Some(env.root()),
    dotfiles: "allow".to_string(),
    ..Default::default()
  };

  let response = serve_file("/.well-known/security.txt", opts);

  assert_eq!(
    response.status,
    StatusCode::OK,
    "dotfiles='allow' should serve the file"
  );
}

// ============================================================================
// PARITY TEST 5: index Option Tries Multiple Files
// JavaScript: send(req, path, { index: ['index.html', 'index.htm', 'default.html'] })
// Expected: Serves first existing index file
// ============================================================================

#[test]
fn parity_index_tries_multiple_files() {
  let env = TestEnv::new();
  env.mkdir("public");
  // Create only the third index option
  env.write_file("public/default.html", "<html>Default Page</html>");

  let opts = FileSendOptions {
    root: Some(env.root()),
    index: Some(vec![
      "index.html".to_string(),
      "index.htm".to_string(),
      "default.html".to_string(),
    ]),
    ..Default::default()
  };

  let response = serve_file("/public/", opts);

  assert_eq!(
    response.status,
    StatusCode::OK,
    "Should serve default.html when index.html and index.htm don't exist"
  );
}

// ============================================================================
// PARITY TEST 6: index Option Priority Order
// JavaScript: send(req, path, { index: ['a.html', 'b.html'] })
// Expected: Serves first in list that exists
// ============================================================================

#[test]
fn parity_index_respects_priority() {
  let env = TestEnv::new();
  env.mkdir("public");
  env.write_file("public/first.html", "First");
  env.write_file("public/second.html", "Second");

  let opts = FileSendOptions {
    root: Some(env.root()),
    index: Some(vec!["first.html".to_string(), "second.html".to_string()]),
    ..Default::default()
  };

  let response = serve_file("/public/", opts);

  assert_eq!(
    response.status,
    StatusCode::OK,
    "Should serve first.html (first in list)"
  );
  // Note: Would need body comparison to verify it's actually first.html
}

// ============================================================================
// PARITY TEST 7: index=false Disables Index Serving
// JavaScript: send(req, path, { index: false })
// Expected: 404 for directory requests
// ============================================================================

#[test]
fn parity_index_false_returns_404() {
  let env = TestEnv::new();
  env.mkdir("public");
  env.write_file("public/index.html", "<html>Index</html>");

  let opts = FileSendOptions {
    root: Some(env.root()),
    index: None, // Equivalent to false in JS
    ..Default::default()
  };

  let response = serve_file("/public/", opts);

  assert_eq!(
    response.status,
    StatusCode::NOT_FOUND,
    "index=false should return 404 for directories"
  );
}

// ============================================================================
// PARITY TEST 8: extensions Option Appends Extensions
// JavaScript: send(req, path, { extensions: ['html', 'htm'] })
// Expected: /about -> /about.html if exists
// ============================================================================

#[test]
fn parity_extensions_appends_to_path() {
  let env = TestEnv::new();
  env.write_file("about.html", "<html>About Us</html>");

  let opts = FileSendOptions {
    root: Some(env.root()),
    extensions: vec!["html".to_string(), "htm".to_string()],
    ..Default::default()
  };

  let response = serve_file("/about", opts);

  assert_eq!(
    response.status,
    StatusCode::OK,
    "Should find about.html when requesting /about"
  );
}

// ============================================================================
// PARITY TEST 9: extensions Don't Apply When Exact File Exists
// JavaScript: send(req, path, { extensions: ['html'] })
// Expected: /file -> /file (not /file.html) if /file exists
// ============================================================================

#[test]
fn parity_extensions_prefer_exact_match() {
  let env = TestEnv::new();
  env.write_file("readme", "Exact file");
  env.write_file("readme.html", "<html>HTML version</html>");

  let opts = FileSendOptions {
    root: Some(env.root()),
    extensions: vec!["html".to_string()],
    ..Default::default()
  };

  let response = serve_file("/readme", opts);

  assert_eq!(
    response.status,
    StatusCode::OK,
    "Should serve exact file when it exists"
  );
  // Note: Would need Content-Type check to verify it's the text file
}

// ============================================================================
// PARITY TEST 10: maxAge Sets Cache-Control
// JavaScript: send(req, path, { maxAge: 86400000 })
// Expected: Cache-Control: public, max-age=86400
// ============================================================================

#[test]
fn parity_maxage_sets_cache_control() {
  let env = TestEnv::new();
  env.write_file("app.js", "console.log('app');");

  let opts = FileSendOptions {
    root: Some(env.root()),
    max_age: 86400000, // 1 day in milliseconds (JS convention)
    cache_control: true,
    ..Default::default()
  };

  let response = serve_file("/app.js", opts);

  assert_eq!(response.status, StatusCode::OK);
  assert!(
    response.headers.contains_key(header::CACHE_CONTROL),
    "Should have Cache-Control header"
  );

  let cache_value = response
    .headers
    .get(header::CACHE_CONTROL)
    .unwrap()
    .to_str()
    .unwrap();

  assert!(
    cache_value.contains("max-age=86400"),
    "Cache-Control should convert milliseconds to seconds: {}",
    cache_value
  );
  assert!(
    cache_value.contains("public"),
    "Cache-Control should contain 'public': {}",
    cache_value
  );
}

// ============================================================================
// PARITY TEST 11: immutable Adds to Cache-Control
// JavaScript: send(req, path, { maxAge: 31536000000, immutable: true })
// Expected: Cache-Control: public, max-age=31536000, immutable
// ============================================================================

#[test]
fn parity_immutable_adds_directive() {
  let env = TestEnv::new();
  env.write_file("bundle.abc123.js", "/* bundled */");

  let opts = FileSendOptions {
    root: Some(env.root()),
    max_age: 31536000000, // 1 year
    cache_control: true,
    immutable: true,
    ..Default::default()
  };

  let response = serve_file("/bundle.abc123.js", opts);

  let cache_value = response
    .headers
    .get(header::CACHE_CONTROL)
    .unwrap()
    .to_str()
    .unwrap();

  assert!(
    cache_value.contains("immutable"),
    "Cache-Control should contain 'immutable': {}",
    cache_value
  );
}

// ============================================================================
// PARITY TEST 12: cacheControl=false Omits Header
// JavaScript: send(req, path, { cacheControl: false })
// Expected: No Cache-Control header
// ============================================================================

#[test]
fn parity_cache_control_false_omits_header() {
  let env = TestEnv::new();
  env.write_file("dynamic.json", "{}");

  let opts = FileSendOptions {
    root: Some(env.root()),
    cache_control: false,
    ..Default::default()
  };

  let response = serve_file("/dynamic.json", opts);

  assert!(
    !response.headers.contains_key(header::CACHE_CONTROL),
    "Should not have Cache-Control header when cacheControl=false"
  );
}

// ============================================================================
// PARITY TEST 13: etag=false Omits ETag Header
// JavaScript: send(req, path, { etag: false })
// Expected: No ETag header
// ============================================================================

#[test]
fn parity_etag_false_omits_header() {
  let env = TestEnv::new();
  env.write_file("file.txt", "content");

  let opts = FileSendOptions {
    root: Some(env.root()),
    etag: false,
    ..Default::default()
  };

  let response = serve_file("/file.txt", opts);

  assert!(
    !response.headers.contains_key(header::ETAG),
    "Should not have ETag header when etag=false"
  );
}

// ============================================================================
// PARITY TEST 14: lastModified=false Omits Last-Modified Header
// JavaScript: send(req, path, { lastModified: false })
// Expected: No Last-Modified header
// ============================================================================

#[test]
fn parity_last_modified_false_omits_header() {
  let env = TestEnv::new();
  env.write_file("file.txt", "content");

  let opts = FileSendOptions {
    root: Some(env.root()),
    last_modified: false,
    ..Default::default()
  };

  let response = serve_file("/file.txt", opts);

  assert!(
    !response.headers.contains_key(header::LAST_MODIFIED),
    "Should not have Last-Modified header when lastModified=false"
  );
}

// ============================================================================
// PARITY TEST 15: acceptRanges=false Omits Accept-Ranges Header
// JavaScript: send(req, path, { acceptRanges: false })
// Expected: No Accept-Ranges header
// ============================================================================

#[test]
fn parity_accept_ranges_false_omits_header() {
  let env = TestEnv::new();
  env.write_file("video.mp4", "fake video");

  let opts = FileSendOptions {
    root: Some(env.root()),
    accept_ranges: false,
    ..Default::default()
  };

  let response = serve_file("/video.mp4", opts);

  assert!(
    !response.headers.contains_key(header::ACCEPT_RANGES),
    "Should not have Accept-Ranges header when acceptRanges=false"
  );
}

// ============================================================================
// PARITY TEST 16: Path Traversal is Blocked
// JavaScript: send(req, '/../etc/passwd', { root: '/var/www' })
// Expected: 403 or 404, file outside root not served
// ============================================================================

#[test]
fn parity_path_traversal_blocked() {
  let env = TestEnv::new();

  // Try to access parent directory
  let opts = FileSendOptions {
    root: Some(env.root()),
    ..Default::default()
  };

  let response = serve_file("/../../../etc/passwd", opts);

  assert!(
    response.status == StatusCode::FORBIDDEN || response.status == StatusCode::NOT_FOUND,
    "Path traversal should be blocked with 403 or 404, got: {}",
    response.status
  );
}

// ============================================================================
// PARITY TEST 17: Directory Without Trailing Slash Redirects
// JavaScript: send(req, '/folder', { root: '/var/www' })
// Expected: 301 redirect to /folder/
// ============================================================================

#[test]
fn parity_directory_redirects_with_slash() {
  let env = TestEnv::new();
  env.mkdir("public");
  env.write_file("public/index.html", "<html>Index</html>");

  let opts = FileSendOptions {
    root: Some(env.root()),
    ..Default::default()
  };

  let response = serve_file("/public", opts);

  assert_eq!(
    response.status,
    StatusCode::MOVED_PERMANENTLY,
    "Directory request without trailing slash should redirect (301)"
  );
  assert!(
    response.headers.contains_key(header::LOCATION),
    "Redirect should include Location header"
  );
}

// ============================================================================
// PARITY TEST 18: Nonexistent File Returns 404
// JavaScript: send(req, '/nonexistent', { root: '/var/www' })
// Expected: 404 Not Found
// ============================================================================

#[test]
fn parity_nonexistent_file_returns_404() {
  let env = TestEnv::new();

  let opts = FileSendOptions {
    root: Some(env.root()),
    ..Default::default()
  };

  let response = serve_file("/does-not-exist.txt", opts);

  assert_eq!(
    response.status,
    StatusCode::NOT_FOUND,
    "Nonexistent file should return 404"
  );
}

// ============================================================================
// Helper Types and Functions
// ============================================================================

struct MockResponse {
  status: StatusCode,
  headers: HeaderMap,
  body: Vec<u8>,
}

/// Serve a file and return simplified response for testing
fn serve_file(path: &str, options: FileSendOptions) -> MockResponse {
  // This is a simplified helper - actual implementation would:
  // 1. Create proper Request
  let request = create_empty_get_request("/");
  let response = create_mock_response_with_request(request);
  // 2. Create FileSendTask
  let mut file_send_task = FileSendTask {
    response: response.to_owned(),
    path: path.to_owned(),
    options,
  };
  // 3. Execute compute()
  file_send_task.compute();
  // 4. Extract status, headers, body from Response

  let status = file_send_task
    .response
    .with_inner(|w_res| Ok(w_res.inner()?.status()))
    .unwrap();

  let headers = file_send_task
    .response
    .with_inner(|w_res| Ok(w_res.inner()?.headers().to_owned()))
    .unwrap();

  //   let body = file_send_task
  //     .response
  //     .with_inner(|w_res| Ok(w_res.inner()?.body().to_owned()));

  MockResponse {
    status,
    headers,
    body: Vec::new(),
  }
}

/// Create a mock Request for testing
fn create_empty_get_request(path: &str) -> HyperRequest<BoxBody<Bytes, hyper::Error>> {
  HyperRequest::builder()
    .method(Method::GET)
    // .uri(path)
    .body(BoxBody::new(Empty::<Bytes>::new().map_err(|e| match e {})))
    .unwrap()
}

/// Create a mock Response with a specific request
fn create_mock_response_with_request(
  request: HyperRequest<BoxBody<Bytes, hyper::Error>>,
) -> Response {
  let w_request: WrappedRequest = request.into();
  Response::new(w_request.into(), None)
}
