mod create_html_document;
mod options;

use std::{
  fs::{self, Metadata},
  path::Path,
  sync::LazyLock,
};

use bytes::Bytes;
use etag::EntityTag;
use headers_core::{HeaderName, HeaderValue};
use hyper::{
  HeaderMap, Method, StatusCode,
  header::{
    self, ACCEPT_RANGES, CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, ETAG, LAST_MODIFIED,
    X_CONTENT_TYPE_OPTIONS,
  },
};
use hyper_staticfile::{AcceptEncoding, ResolveResult, Resolver};
use napi::bindgen_prelude::*;
use regex::Regex;
use tokio::runtime::Runtime;

use crate::{
  response::Response,
  utilities::{self, file_send_task::create_html_document::create_html_document},
};
pub use options::FileSendOptions;

static UP_PATH_REGEXP: LazyLock<Regex> =
  LazyLock::new(|| Regex::new(r"(?:^|[/\\])\.\.(?:[/\\]|$)").unwrap());

pub struct FileSendTask {
  pub response: Response,
  pub path: String,
  pub options: FileSendOptions,
}

impl FileSendTask {
  fn error(&self, status: StatusCode, headers: Option<HeaderMap>) -> Result<()> {
    let msg = status.canonical_reason().unwrap_or(status.as_str());
    let doc = create_html_document("Error".to_owned(), ammonia::clean(msg));

    self.response.with_inner(|w_res| {
      let inner = w_res.inner()?;
      *inner.status_mut() = status;

      let res_headers = inner.headers_mut();

      // clear existing headers
      res_headers.clear();

      // add error headers
      if let Some(headers) = headers {
        res_headers.extend(headers);
      }

      res_headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_static("text/html; charset=UTF-8"),
      );
      res_headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static("default-src 'none'"),
      );
      res_headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));

      w_res.end(Some(Bytes::copy_from_slice(doc.as_bytes())))
    })
  }

  fn set_header(&self, stat: &Metadata) -> Result<()> {
    self.response.with_inner(|w_res| {
      let headers = w_res.inner()?.headers_mut();

      if self.options.accept_ranges && !headers.contains_key(ACCEPT_RANGES) {
        headers.insert(ACCEPT_RANGES, HeaderValue::from_static("bytes"));
      }

      if self.options.cache_control && !headers.contains_key(CACHE_CONTROL) {
        let max_age_secs = self.options.max_age / 1000;
        let cache_control = if self.options.immutable {
          format!("public, max-age={}, immutable", max_age_secs)
        } else {
          format!("public, max-age={}", max_age_secs)
        };

        if let Ok(value) = HeaderValue::from_str(&cache_control) {
          headers.insert(CACHE_CONTROL, value);
        }
      }

      if self.options.last_modified
        && !headers.contains_key(LAST_MODIFIED)
        && let Ok(modified) = stat.modified()
      {
        let datetime: chrono::DateTime<chrono::Utc> = modified.into();
        let modified_str = datetime.format("%a, %d %b %Y %H:%M:%S GMT").to_string();

        if let Ok(value) = HeaderValue::from_str(&modified_str) {
          headers.insert(LAST_MODIFIED, value);
        }
      }

      if self.options.etag && !headers.contains_key(ETAG) {
        let etag_val = EntityTag::from_file_meta(stat);
        if let Ok(value) = HeaderValue::from_str(etag_val.tag()) {
          headers.insert(ETAG, value);
        }
      }

      Ok(())
    })
  }
}

impl Task for FileSendTask {
  type Output = ();
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    if self.path.is_empty() {
      self.error(StatusCode::BAD_REQUEST, None);
      return Ok(());
    }

    // null byte(s)
    if self.path.contains('\0') {
      self.error(StatusCode::BAD_REQUEST, None);
      return Ok(());
    }

    let mut path = Path::new(&self.path).to_owned();

    match self.options.root.is_some() {
      true => {
        path = std::path::absolute(path).map_err(|e| {
          Error::new(
            Status::GenericFailure,
            format!("Error making path absolute: {e}"),
          )
        })?;

        // malicious path
        if UP_PATH_REGEXP.is_match(&path.display().to_string()) {
          println!("malicious path '{path:?}'");
          self.error(StatusCode::FORBIDDEN, None);
          return Ok(());
        }
      }
      false => {
        // ".." is malicious without "root"
        if UP_PATH_REGEXP.is_match(&path.display().to_string()) {
          println!("malicious path '{path:?}'");
          self.error(StatusCode::FORBIDDEN, None);
          return Ok(());
        }
      }
    }

    // dotfile handling
    if utilities::contains_dot_file(path.as_path()) {
      match self.options.dotfiles.as_str() {
        "allow" => {}
        "deny" => {
          self.error(StatusCode::FORBIDDEN, None);
          return Ok(());
        }
        _ => {
          self.error(StatusCode::NOT_FOUND, None);
          return Ok(());
        }
      }
    }

    let root = match &self.options.root {
      Some(root) => root.to_owned(),
      None => std::env::current_dir().map_err(|e| {
        Error::new(
          Status::GenericFailure,
          format!("Error accessing the current directory: {e}"),
        )
      })?,
    };

    let Ok(mut relative_path) = path.strip_prefix(&root) else {
      self.error(StatusCode::FORBIDDEN, None);
      return Ok(());
    };
    if path.is_dir() {
      match self.options.index.as_ref() {
        Some(index_files) => {
          let first_found_index_file = index_files.iter().find(|index_file| {
            let index_file_path = path.join(index_file);
            index_file_path.is_file()
          });

          match first_found_index_file {
            Some(index_file_path) => {
              path = path.join(index_file_path);
              relative_path = Path::new(index_file_path);
            }
            // none of the specified index files exist
            None => {
              self.error(StatusCode::NOT_FOUND, None);
              return Ok(());
            }
          }
        }
        // serve index file disabled
        None => {
          self.error(StatusCode::NOT_FOUND, None);
          return Ok(());
        }
      }
    }

    let file_metadata = match fs::metadata(&path) {
      Ok(metadata) => metadata,
      Err(e) => {
        log::error!("{e}");
        self.error(StatusCode::INTERNAL_SERVER_ERROR, None);
        return Ok(());
      }
    };
    if let Err(e) = self.set_header(&file_metadata) {
      log::error!("{e}");
      self.error(StatusCode::INTERNAL_SERVER_ERROR, None);
      return Ok(());
    }

    // Create the runtime
    let rt = Runtime::new()?;

    // Get a handle from this runtime
    let handle = rt.handle();

    let request = self
      .response
      .req()
      .with_inner_mut(|w_req| w_req.take_inner())?;

    // Handle only `GET`/`HEAD` and absolute paths.
    let result = match *request.method() {
      Method::HEAD | Method::GET => {
        let resolver = Resolver::new(&root);

        // Parse `Accept-Encoding` header.
        let accept_encoding = resolver.allowed_encodings
          & request
            .headers()
            .get(header::ACCEPT_ENCODING)
            .map(AcceptEncoding::from_header_value)
            .unwrap_or(AcceptEncoding::none());

        println!(
          "RS: Resolving path '{}' in {:?}",
          relative_path.display(),
          root
        );

        let Some(relative_path) = relative_path.to_str() else {
          log::error!("Support for non-UTF-8 paths not yet implemented!");
          self.error(StatusCode::INTERNAL_SERVER_ERROR, None);
          return Ok(());
        };

        handle.block_on(async { resolver.resolve_path(relative_path, accept_encoding).await })?
      }
      _ => ResolveResult::MethodNotMatched,
    };

    println!("RS: Result: {result:?}");

    let mut response = hyper_staticfile::ResponseBuilder::new()
      .request(&request)
      .build(result)
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?
      .map(|b| b.into());

    if let Some(headers) = &self.options.headers {
      let mut previous_header: Option<HeaderName> = None;
      #[allow(clippy::unnecessary_to_owned)]
      for (name, value) in headers.to_owned().into_iter() {
        if let Some(name) = name {
          response.headers_mut().insert(name.to_owned(), value);
          previous_header = Some(name);
        } else if let Some(name) = &previous_header {
          response.headers_mut().append(name, value);
        }
      }
    }

    self.response.req().with_inner_mut(|w_req| {
      w_req.set_inner(request);
      Ok(())
    })?;

    self.response.with_inner(|w_res| {
      w_res.set_inner(response);
      Ok(())
    })
  }

  fn resolve(&mut self, _env: Env, _compute_output: Self::Output) -> Result<Self::JsValue> {
    Ok(())
  }
}
