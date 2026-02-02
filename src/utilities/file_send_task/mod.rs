mod options;

use std::{
  fs::{self, Metadata},
  path::{Path, PathBuf},
};

use etag::EntityTag;
use headers_core::{HeaderName, HeaderValue};
use hyper::{
  Method, Response as HyperResponse, StatusCode,
  header::{self, ACCEPT_RANGES, CACHE_CONTROL, ETAG, LAST_MODIFIED},
};
use hyper_staticfile::{AcceptEncoding, ResolveResult, Resolver};
use napi::bindgen_prelude::*;
use tokio::runtime::Runtime;

use crate::{
  response::{CrateBody, Response},
  utilities,
};
pub use options::FileSendOptions;

pub struct FileSendTask {
  pub response: Response,
  pub root: PathBuf,
  pub path: String,
  pub options: FileSendOptions,
  pub etag: bool,
}

impl FileSendTask {
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

      if self.etag && !headers.contains_key(ETAG) {
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
  type Output = HyperResponse<CrateBody>;
  type JsValue = ();

  fn compute(&mut self) -> Result<Self::Output> {
    // dotfile handling
    if utilities::contains_dot_file(Path::new(&self.path)) {
      match self.options.dotfiles.as_str() {
        "allow" => {}
        "deny" => {
          // return 403 response
          let mut response = HyperResponse::new(CrateBody::Empty);
          *response.status_mut() = StatusCode::FORBIDDEN;
          return Ok(response);
        }
        _ => {
          // return 404 response
          let mut response = HyperResponse::new(CrateBody::Empty);
          *response.status_mut() = StatusCode::NOT_FOUND;
          return Ok(response);
        }
      }
    }

    let absolute_path = Path::new(&self.root).join(&self.path);
    let mut relative_path = &self.path;
    if absolute_path.is_dir() {
      match self.options.index.as_ref() {
        Some(index_files) => {
          let first_found_index_file = index_files.iter().find(|index_file| {
            let index_file_path = Path::new(&self.root).join(index_file);
            index_file_path.is_file()
          });

          if let Some(index_file_path) = first_found_index_file {
            relative_path = index_file_path
          }
        }
        // serve index file disabled
        None => {
          let mut response = HyperResponse::new(CrateBody::Empty);
          *response.status_mut() = StatusCode::NOT_FOUND;
          return Ok(response);
        }
      }
    }

    let file_metadata = match fs::metadata(absolute_path) {
      Ok(metadata) => metadata,
      Err(e) => {
        log::error!("{e}");
        let mut response = HyperResponse::new(CrateBody::Empty);
        *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
        return Ok(response);
      }
    };
    if let Err(e) = self.set_header(&file_metadata) {
      log::error!("{e}");
      let mut response = HyperResponse::new(CrateBody::Empty);
      *response.status_mut() = StatusCode::INTERNAL_SERVER_ERROR;
      return Ok(response);
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
        let resolver = Resolver::new(&self.root);

        // Parse `Accept-Encoding` header.
        let accept_encoding = resolver.allowed_encodings
          & request
            .headers()
            .get(header::ACCEPT_ENCODING)
            .map(AcceptEncoding::from_header_value)
            .unwrap_or(AcceptEncoding::none());

        println!("RS: Resolving path '{}' in {:?}", relative_path, self.root);

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

    Ok(response)
  }

  fn resolve(&mut self, _env: Env, response: Self::Output) -> Result<Self::JsValue> {
    self.response.with_inner(|w_res| {
      w_res.set_inner(response);
      Ok(())
    })
  }
}
