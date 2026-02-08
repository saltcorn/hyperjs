use std::path::PathBuf;

use headers_core::HeaderValue;
use hyper::{
  StatusCode,
  header::{ALLOW, CONTENT_LENGTH},
};
use napi::bindgen_prelude::*;

use super::StaticOptions;
use crate::{
  request::Request,
  response::Response,
  utilities::{FileSendTask, parse_url::RequestExt},
};

pub struct StaticMiddlewareTask {
  pub response: Response,
  pub request: Request,
  pub root: PathBuf,
  pub options: StaticOptions,
}

impl Task for StaticMiddlewareTask {
  type Output = bool;
  type JsValue = bool;

  fn compute(&mut self) -> Result<Self::Output> {
    let request_method = self.request.method()?;
    if request_method.as_str() != "GET" && request_method.as_str() != "HEAD" {
      if self.options.fallthrough {
        return Ok(true);
      }

      self.response.with_inner(|w_res| {
        let inner = w_res.inner()?;
        *inner.status_mut() = StatusCode::METHOD_NOT_ALLOWED;
        let headers = inner.headers_mut();
        headers.insert(ALLOW, HeaderValue::from_static("GET, HEAD"));
        headers.insert(CONTENT_LENGTH, HeaderValue::from_static("0"));
        Ok(())
      })?;

      self.response.end(None)?;
      return Ok(false);
    }

    // TODO: Enhance to match static middleware behavior
    // let forward_error = !self.options.fallthrough;
    let (original_url, parsed_url) = self.request.with_inner(|w_req| {
      let inner = w_req.inner()?;
      Ok((inner.original_url(), inner.parseurl()))
    })?;
    let original_url = original_url.ok_or_else(|| {
      Error::new(
        Status::GenericFailure,
        "Expected request's original URL to be set.",
      )
    })?;
    let mut path = parsed_url
      .ok_or_else(|| Error::new(Status::GenericFailure, "Expected request's URL to be set."))?
      .pathname;

    // make sure redirect occurs at mount
    if path.as_str() == "/" && original_url.pathname.ends_with('/') {
      path = String::with_capacity(0);
    }

    let mut file_send_task = FileSendTask {
      response: self.response.to_owned(),
      path,
      options: (&self.options).into(),
    };

    file_send_task.compute()?;

    Ok(true)
  }

  fn resolve(&mut self, _env: Env, compute_output: Self::Output) -> Result<Self::JsValue> {
    Ok(compute_output)
  }
}
