#[cfg(test)]
mod file_send_task_tests;
#[cfg(test)]
mod parity_tests;
mod requested_path;

use std::{
  fs::Metadata,
  path::{Component, Path, PathBuf},
  sync::{Arc, Mutex},
  time::Instant,
};

use hyper::{
  HeaderMap, StatusCode,
  body::Bytes,
  header::{self, HeaderValue},
};
use napi::bindgen_prelude::*;
use tokio::runtime::Runtime;

use crate::response::Response;
use requested_path::RequestedPath;

/// Configuration options for file serving (matching JavaScript send() options)
#[derive(Clone, Debug)]
pub struct FileSendOptions {
  /// Root directory to serve files from
  pub root: Option<PathBuf>,

  /// Maximum age for caching in milliseconds
  pub max_age: u64,

  /// Whether to send Cache-Control header
  pub cache_control: bool,

  /// Whether to send ETag header (always true in hyper-staticfile)
  pub etag: bool,

  /// Whether to send Last-Modified header (always true in hyper-staticfile)
  pub last_modified: bool,

  /// Whether to accept range requests (always true in hyper-staticfile)
  pub accept_ranges: bool,

  /// Whether to add 'immutable' directive to Cache-Control
  pub immutable: bool,

  /// Index file names to try for directories
  pub index: Option<Vec<String>>,

  /// File extensions to try if file not found
  pub extensions: Option<Vec<String>>,

  /// Dotfile handling: "allow", "deny", or "ignore"
  pub dotfiles: String,

  /// Custom headers to add to response
  pub headers: Option<HeaderMap>,
}

impl Default for FileSendOptions {
  fn default() -> Self {
    Self {
      root: None,
      max_age: 0,
      cache_control: true,
      etag: true,
      last_modified: true,
      accept_ranges: true,
      immutable: false,
      index: Some(vec!["index.html".to_string()]),
      extensions: None,
      dotfiles: "ignore".to_string(),
      headers: None,
    }
  }
}

pub struct FileSendTask {
  pub response: Response,
  pub path: String,
  pub options: FileSendOptions,
}

impl FileSendTask {
  /// Check if path contains a dotfile component
  fn contains_dotfile(path: &Path) -> bool {
    path.components().any(|c| {
      if let Component::Normal(name) = c
        && let Some(s) = name.to_str()
      {
        return s.starts_with('.') && s.len() > 1;
      }
      false
    })
  }

  /// Build error response matching JavaScript send() behavior
  fn error(&self, status: StatusCode, headers: Option<HeaderMap>) -> Result<()> {
    let msg = status.canonical_reason().unwrap_or(status.as_str());
    let doc = create_html_document("Error", ammonia::clean(msg));

    self.response.with_inner(|w_res| {
      let inner = w_res.inner()?;
      *inner.status_mut() = status;

      let res_headers = inner.headers_mut();
      res_headers.clear();

      if let Some(headers) = headers {
        res_headers.extend(headers);
      }

      res_headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=UTF-8"),
      );
      res_headers.insert(
        header::CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'"),
      );
      res_headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
      );

      w_res.end(Some(Bytes::copy_from_slice(doc.as_bytes())))
    })
  }
}

pub struct FileServeResult {
  pub served_path: PathBuf,
  pub file_stat: Metadata,
}

impl Task for FileSendTask {
  type Output = Option<FileServeResult>;
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    // Get the root directory
    let root = match &self.options.root {
      Some(root) => root.clone(),
      None => std::env::current_dir().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Error accessing current directory: {e}"),
        )
      })?,
    };

    // Create Tokio runtime
    let rt = Runtime::new()?;

    // Extract request from wrapper
    let mut request = self
      .response
      .req()
      .with_inner_mut(|w_req| w_req.take_inner())?;
    let request_path = if !self.path.starts_with("/") {
      let mut p = String::from("/");
      p.push_str(&self.path);
      p
    } else {
      self.path.to_owned()
    };
    match request_path.parse() {
      Ok(path) => {
        *request.uri_mut() = path;
      }
      Err(e) => {
        log::error!("{e}");
        self.error(StatusCode::FORBIDDEN, None)?;
        return Ok(None);
      }
    }

    // Create resolver with all configuration
    let mut resolver = hyper_staticfile::Resolver::new(&root);

    // Enable all encodings (gzip, brotli, zstd)
    resolver.allowed_encodings = hyper_staticfile::AcceptEncoding::all();

    // 1. Handle dotfiles

    let requested_path = RequestedPath::resolve(&request_path);
    let sanitized_path = requested_path.sanitized;
    if Self::contains_dotfile(&sanitized_path) {
      match self.options.dotfiles.as_str() {
        "allow" => {
          // Allow dotfiles - continue processing
        }
        "deny" => {
          // Return 403 Forbidden
          self.error(StatusCode::FORBIDDEN, None)?;
          return Ok(None);
        }
        _ => {
          // "ignore" or anything else - return 404 Not Found
          self.error(StatusCode::NOT_FOUND, None)?;
          return Ok(None);
        }
      }
    }

    // Configure rewrite hook for dotfiles, index, and extensions
    let options_clone = self.options.clone();
    let root_clone = root.clone();
    let file_serve_result: Arc<Mutex<Option<FileServeResult>>> = Default::default();
    let file_serve_result_clone = file_serve_result.clone();

    resolver.set_rewrite(move |mut params| {
      let options = options_clone.clone();
      let root = root_clone.clone();
      let instant = Instant::now();
      let file_serve_result = file_serve_result_clone.clone();

      async move {
        // 2. Handle index files for directories
        if params.is_dir_request {
          match options.index {
            Some(ref index_files) => {
              for index_name in index_files {
                let mut test_path = root.clone();
                test_path.push(&params.path);
                test_path.push(index_name);

                // Check if file exists using tokio::fs
                if let Ok(metadata) = tokio::fs::metadata(&test_path).await
                  && metadata.is_file()
                {
                  params.path.push(index_name);
                  params.is_dir_request = false;
                  {
                    let mut file_serve_result = file_serve_result.lock().map_err(|_| {
                      std::io::Error::other("failed to obtain lock on file_serve_result: {e}")
                    })?;
                    let _ = file_serve_result.insert(FileServeResult {
                      served_path: params.path.to_owned(),
                      file_stat: metadata,
                    });
                  }
                  return Ok(params);
                }
              }
            }
            None => {
              let improbable_name = instant.elapsed().as_nanos().to_string();
              params.path.push(improbable_name);
              params.is_dir_request = false;
              return Ok(params);
            }
          }
          // If no index found, let hyper-staticfile handle it (will return 404)
          return Ok(params);
        }

        // 3. Handle extension fallback
        if let Some(extensions) = &options.extensions {
          let mut test_path = root.clone();
          test_path.push(&params.path);

          // Check if original path exists
          if tokio::fs::metadata(&test_path).await.is_err() {
            // Try each extension
            for ext in extensions {
              let mut ext_path = test_path.clone().into_os_string();
              ext_path.push(".");
              ext_path.push(ext);
              let ext_path: PathBuf = ext_path.into();

              if let Ok(metadata) = tokio::fs::metadata(&ext_path).await
                && metadata.is_file()
              {
                // Found file with extension
                let mut new_path = params.path.clone().into_os_string();
                new_path.push(".");
                new_path.push(ext);
                params.path = new_path.into();
                {
                  let mut file_serve_result = file_serve_result.lock().map_err(|_| {
                    std::io::Error::other("failed to obtain lock on file_serve_result: {e}")
                  })?;
                  let _ = file_serve_result.insert(FileServeResult {
                    served_path: params.path.to_owned(),
                    file_stat: metadata,
                  });
                }
                return Ok(params);
              }
            }
          }
        }

        Ok(params)
      }
    });

    // Resolve the request using hyper-staticfile
    let result = rt.block_on(async { resolver.resolve_request(&request).await })?;

    // Build response with cache headers
    let cache_headers = if self.options.cache_control {
      // Convert milliseconds to seconds
      Some((self.options.max_age / 1000) as u32)
    } else {
      None
    };

    let mut response = hyper_staticfile::ResponseBuilder::new()
      .request(&request)
      .cache_headers(cache_headers)
      .build(result)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?
      .map(|b| b.into());

    // Post-process response headers

    // Add 'immutable' directive to Cache-Control if needed
    if self.options.immutable
      && self.options.cache_control
      && self.options.max_age > 0
      && let Some(cache_control) = response.headers().get(header::CACHE_CONTROL)
      && let Ok(value) = cache_control.to_str()
    {
      let new_value = format!("{}, immutable", value);
      if let Ok(header_value) = HeaderValue::from_str(&new_value) {
        response
          .headers_mut()
          .insert(header::CACHE_CONTROL, header_value);
      }
    }

    // Remove ETag if disabled (though hyper-staticfile always generates it)
    if !self.options.etag {
      response.headers_mut().remove(header::ETAG);
    }

    // Remove Last-Modified if disabled
    if !self.options.last_modified {
      response.headers_mut().remove(header::LAST_MODIFIED);
    }

    // Remove Accept-Ranges if disabled
    if !self.options.accept_ranges {
      response.headers_mut().remove(header::ACCEPT_RANGES);
    }

    // Add custom headers
    if let Some(ref headers) = self.options.headers {
      let mut previous_header: Option<header::HeaderName> = None;

      for (name, value) in headers.clone().into_iter() {
        if let Some(name) = name {
          response.headers_mut().insert(name.clone(), value.clone());
          previous_header = Some(name.clone());
        } else if let Some(name) = &previous_header {
          response.headers_mut().append(name, value.clone());
        }
      }
    }

    // Put request back
    self.response.req().with_inner_mut(|w_req| {
      w_req.set_inner(request);
      Ok(())
    })?;

    // Set response
    self.response.with_inner(|w_res| {
      w_res.set_inner(response);
      Ok(())
    })?;

    let file_serve_result = {
      file_serve_result
        .lock()
        .map_err(|_| std::io::Error::other("failed to obtain lock on file_serve_result: {e}"))?
        .take()
    };

    Ok(file_serve_result)
  }

  fn resolve(&mut self, _env: Env, _output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }
}

fn create_html_document(title: &str, body: String) -> String {
  format!(
    "<!DOCTYPE html>\n\
         <html lang=\"en\">\n\
         <head>\n\
         <meta charset=\"utf-8\">\n\
         <title>{}</title>\n\
         </head>\n\
         <body>\n\
         <pre>{}</pre>\n\
         </body>\n\
         </html>\n",
    title, body
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_contains_dotfile() {
    assert!(FileSendTask::contains_dotfile(&PathBuf::from(
      ".git/config"
    )));
    assert!(FileSendTask::contains_dotfile(&PathBuf::from(
      "path/.hidden/file"
    )));
    assert!(FileSendTask::contains_dotfile(&PathBuf::from(".dotfile")));

    assert!(!FileSendTask::contains_dotfile(&PathBuf::from(
      "normal/path"
    )));
    assert!(!FileSendTask::contains_dotfile(&PathBuf::from(
      "path/file.txt"
    )));
    assert!(!FileSendTask::contains_dotfile(&PathBuf::from(".")));
  }

  #[test]
  fn test_default_options() {
    let opts = FileSendOptions::default();

    assert_eq!(opts.max_age, 0);
    assert!(opts.cache_control);
    assert!(opts.etag);
    assert!(opts.last_modified);
    assert!(opts.accept_ranges);
    assert!(!opts.immutable);
    assert_eq!(opts.dotfiles, "ignore");
    assert_eq!(opts.index, Some(vec!["index.html".to_string()]));
    assert!(opts.extensions.is_none());
  }

  #[test]
  fn test_create_html_document() {
    let doc = create_html_document("Error", "Not Found".to_string());

    assert!(doc.contains("<!DOCTYPE html>"));
    assert!(doc.contains("<title>Error</title>"));
    assert!(doc.contains("<pre>Not Found</pre>"));
  }
}
