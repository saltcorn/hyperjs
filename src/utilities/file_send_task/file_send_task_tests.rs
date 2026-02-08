use std::{
  fs::{self, File},
  io::Write,
  path::PathBuf,
};

use bytes::Bytes;
use http_body_util::{BodyExt, Empty, combinators::BoxBody};
use hyper::{
  HeaderMap, Method, Request as HyperRequest, StatusCode,
  header::{self, HeaderValue},
};
use napi::bindgen_prelude::Task;
use tempfile::TempDir;

use super::{FileSendOptions, FileSendTask};
use crate::{request::WrappedRequest, response::Response};

/// Test helper to create a test directory structure
struct TestFixture {
  temp_dir: TempDir,
}

impl TestFixture {
  fn new() -> Self {
    let temp_dir = TempDir::new().unwrap();
    Self { temp_dir }
  }

  fn root(&self) -> PathBuf {
    self.temp_dir.path().to_path_buf()
  }

  fn create_file(&self, path: &str, content: &str) {
    let full_path = self.temp_dir.path().join(path);

    // Create parent directories
    if let Some(parent) = full_path.parent() {
      fs::create_dir_all(parent).unwrap();
    }

    let mut file = File::create(full_path).unwrap();
    file.write_all(content.as_bytes()).unwrap();
  }

  fn create_dir(&self, path: &str) {
    let full_path = self.temp_dir.path().join(path);
    fs::create_dir_all(full_path).unwrap();
  }
}

// ============================================================================
// Test 1: Basic File Serving
// ============================================================================

#[test]
fn test_serve_basic_file() {
  let fixture = TestFixture::new();
  fixture.create_file("test.txt", "Hello, World!");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/test.txt".to_owned(),
  };

  task.compute().unwrap();

  // Verify response
  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert_eq!(inner.status(), StatusCode::OK);
      assert!(inner.headers().contains_key(header::CONTENT_TYPE));
      assert!(inner.headers().contains_key(header::ETAG));
      assert!(inner.headers().contains_key(header::LAST_MODIFIED));
      Ok(())
    })
    .unwrap();
}

// ============================================================================
// Test 2: Directory Index Files
// ============================================================================

#[test]
fn test_directory_index_default() {
  let fixture = TestFixture::new();
  fixture.create_dir("public");
  fixture.create_file("public/index.html", "<html>Index</html>");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    index: Some(vec!["index.html".to_string()]),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/public/".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert_eq!(inner.status(), StatusCode::OK);
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_directory_index_multiple() {
  let fixture = TestFixture::new();
  fixture.create_dir("public");
  fixture.create_file("public/default.html", "<html>Default</html>");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    index: Some(vec![
      "index.html".to_string(),
      "index.htm".to_string(),
      "default.html".to_string(),
    ]),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/public/".to_owned(),
  };

  task.compute().unwrap();

  // Should serve default.html since index.html and index.htm don't exist
  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert_eq!(inner.status(), StatusCode::OK);
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_directory_no_index() {
  let fixture = TestFixture::new();
  fixture.create_dir("public");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    index: None, // Disable index serving
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/public/".to_owned(),
  };

  task.compute().unwrap();

  // Should return 404
  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert_eq!(inner.status(), StatusCode::NOT_FOUND);
      Ok(())
    })
    .unwrap();
}

// ============================================================================
// Test 3: Extension Fallback
// ============================================================================

#[test]
fn test_extension_fallback() {
  let fixture = TestFixture::new();
  fixture.create_file("about.html", "<html>About</html>");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    extensions: Some(vec!["html".to_string(), "htm".to_string()]),
    ..Default::default()
  };

  let request = create_empty_get_request(); // Request without extension
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/about".to_owned(),
  };

  task.compute().unwrap();

  // Should find about.html
  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert_eq!(inner.status(), StatusCode::OK);
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_extension_fallback_priority() {
  let fixture = TestFixture::new();
  fixture.create_file("page.html", "HTML version");
  fixture.create_file("page.htm", "HTM version");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    extensions: Some(vec!["html".to_string(), "htm".to_string()]),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/page".to_owned(),
  };

  task.compute().unwrap();

  // Should serve .html first (based on priority order)
  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert_eq!(inner.status(), StatusCode::OK);
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_no_extension_fallback_when_file_exists() {
  let fixture = TestFixture::new();
  fixture.create_file("exact", "Exact file");
  fixture.create_file("exact.html", "HTML file");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    extensions: Some(vec!["html".to_string()]),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/exact".to_owned(),
  };

  task.compute().unwrap();

  // Should serve the exact file, not the .html version
  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert_eq!(inner.status(), StatusCode::OK);
      Ok(())
    })
    .unwrap();
}

// ============================================================================
// Test 4: Dotfile Handling
// ============================================================================

#[test]
fn test_dotfile_allow() {
  let fixture = TestFixture::new();
  fixture.create_file(".secret", "Secret content");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    dotfiles: "allow".to_string(),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/.secret".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert_eq!(inner.status(), StatusCode::OK);
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_dotfile_deny() {
  let fixture = TestFixture::new();
  fixture.create_file(".secret", "Secret content");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    dotfiles: "deny".to_string(),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/.secret".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      // Should return 403 Forbidden
      assert_eq!(inner.status(), StatusCode::FORBIDDEN);
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_dotfile_ignore() {
  let fixture = TestFixture::new();
  fixture.create_file(".secret", "Secret content");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    dotfiles: "ignore".to_string(),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/.secret".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      // Should return 404 Not Found
      assert_eq!(inner.status(), StatusCode::NOT_FOUND);
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_dotfile_in_path() {
  let fixture = TestFixture::new();
  fixture.create_dir(".hidden");
  fixture.create_file(".hidden/file.txt", "Hidden file");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    dotfiles: "deny".to_string(),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/.hidden/file.txt".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert_eq!(inner.status(), StatusCode::FORBIDDEN);
      Ok(())
    })
    .unwrap();
}

// ============================================================================
// Test 5: Cache Control Headers
// ============================================================================

#[test]
fn test_cache_control_enabled() {
  let fixture = TestFixture::new();
  fixture.create_file("test.txt", "Content");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    max_age: 86400000, // 1 day in milliseconds
    cache_control: true,
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/test.txt".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      let cache_control = inner.headers().get(header::CACHE_CONTROL).unwrap();
      let value = cache_control.to_str().unwrap();

      // Should contain max-age=86400 (seconds, not milliseconds)
      assert!(value.contains("max-age=86400"));
      assert!(value.contains("public"));
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_cache_control_disabled() {
  let fixture = TestFixture::new();
  fixture.create_file("test.txt", "Content");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    cache_control: false,
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/test.txt".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      // Should not have Cache-Control header
      assert!(!inner.headers().contains_key(header::CACHE_CONTROL));
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_cache_control_immutable() {
  let fixture = TestFixture::new();
  fixture.create_file("app.js", "// App code");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    max_age: 31536000000, // 1 year
    cache_control: true,
    immutable: true,
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/app.js".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      let cache_control = inner.headers().get(header::CACHE_CONTROL).unwrap();
      let value = cache_control.to_str().unwrap();

      assert!(value.contains("immutable"));
      Ok(())
    })
    .unwrap();
}

// ============================================================================
// Test 6: ETag and Last-Modified Headers
// ============================================================================

#[test]
fn test_etag_header() {
  let fixture = TestFixture::new();
  fixture.create_file("test.txt", "Content");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    etag: true,
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/test.txt".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert!(inner.headers().contains_key(header::ETAG));
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_etag_disabled() {
  let fixture = TestFixture::new();
  fixture.create_file("test.txt", "Content");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    etag: false,
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/test.txt".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert!(!inner.headers().contains_key(header::ETAG));
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_last_modified_header() {
  let fixture = TestFixture::new();
  fixture.create_file("test.txt", "Content");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    last_modified: true,
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/test.txt".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert!(inner.headers().contains_key(header::LAST_MODIFIED));
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_last_modified_disabled() {
  let fixture = TestFixture::new();
  fixture.create_file("test.txt", "Content");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    last_modified: false,
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/test.txt".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert!(!inner.headers().contains_key(header::LAST_MODIFIED));
      Ok(())
    })
    .unwrap();
}

// ============================================================================
// Test 7: Accept-Ranges Header
// ============================================================================

#[test]
fn test_accept_ranges_enabled() {
  let fixture = TestFixture::new();
  fixture.create_file("video.mp4", "fake video data");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    accept_ranges: true,
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/video.mp4".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert!(inner.headers().contains_key(header::ACCEPT_RANGES));
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_accept_ranges_disabled() {
  let fixture = TestFixture::new();
  fixture.create_file("video.mp4", "fake video data");

  let options = FileSendOptions {
    root: Some(fixture.root()),
    accept_ranges: false,
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/video.mp4".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert!(!inner.headers().contains_key(header::ACCEPT_RANGES));
      Ok(())
    })
    .unwrap();
}

// ============================================================================
// Test 8: Custom Headers
// ============================================================================

#[test]
fn test_custom_headers() {
  let fixture = TestFixture::new();
  fixture.create_file("test.txt", "Content");

  let mut custom_headers = HeaderMap::new();
  custom_headers.insert("X-Custom-Header", HeaderValue::from_static("custom-value"));

  let options = FileSendOptions {
    root: Some(fixture.root()),
    headers: Some(custom_headers),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/test.txt".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert!(inner.headers().contains_key("x-custom-header"));
      assert_eq!(
        inner.headers().get("x-custom-header").unwrap(),
        "custom-value"
      );
      Ok(())
    })
    .unwrap();
}

// ============================================================================
// Test 9: Error Handling
// ============================================================================

#[test]
fn test_file_not_found() {
  let fixture = TestFixture::new();

  let options = FileSendOptions {
    root: Some(fixture.root()),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/nonexistent.txt".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      assert_eq!(inner.status(), StatusCode::NOT_FOUND);
      Ok(())
    })
    .unwrap();
}

#[test]
fn test_path_traversal_blocked() {
  let fixture = TestFixture::new();

  // Create a file outside the root
  let root = fixture.root();
  let parent = root.parent().unwrap();
  let secret_path = parent.join("secret.txt");
  fs::write(&secret_path, "Secret!").unwrap();

  let options = FileSendOptions {
    root: Some(fixture.root()),
    ..Default::default()
  };

  let request = create_empty_get_request();
  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/../secret.txt".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      // Should be blocked (404 or 403)
      assert!(inner.status() == StatusCode::NOT_FOUND || inner.status() == StatusCode::FORBIDDEN);
      Ok(())
    })
    .unwrap();

  // Clean up
  fs::remove_file(secret_path).ok();
}

// ============================================================================
// Test 10: Range HyperRequests (from hyper-staticfile)
// ============================================================================

#[test]
fn test_range_request_supported() {
  let fixture = TestFixture::new();
  fixture.create_file("large.bin", "0123456789".repeat(100).as_str());

  let options = FileSendOptions {
    root: Some(fixture.root()),
    accept_ranges: true,
    ..Default::default()
  };

  let mut request_builder = HyperRequest::builder()
    .method(Method::GET)
    .uri("/large.bin");

  // Add Range header
  request_builder = request_builder.header(header::RANGE, "bytes=0-99");

  let request = request_builder
    .body(BoxBody::new(Empty::<Bytes>::new().map_err(|e| match e {})))
    .unwrap();

  let response = create_mock_response_with_request(request);
  let mut task = FileSendTask {
    response,
    options,
    path: "/large.bin".to_owned(),
  };

  task.compute().unwrap();

  task
    .response
    .with_inner(|w_res| {
      let inner = w_res.inner()?;
      // Should return 206 Partial Content
      assert_eq!(inner.status(), StatusCode::PARTIAL_CONTENT);
      assert!(inner.headers().contains_key(header::CONTENT_RANGE));
      Ok(())
    })
    .unwrap();
}

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a mock Request for testing
fn create_empty_get_request() -> HyperRequest<BoxBody<Bytes, hyper::Error>> {
  HyperRequest::builder()
    .method(Method::GET)
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
