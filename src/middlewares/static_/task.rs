use std::path::PathBuf;

use headers_core::HeaderValue;
use hyper::{
  StatusCode,
  header::{ALLOW, CONTENT_LENGTH},
};
use napi::bindgen_prelude::*;
use tokio::runtime::Runtime;

use super::StaticOptions;
use crate::{
  middlewares::static_::FileStat,
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
    if path.as_str() == "/" && !original_url.pathname.ends_with('/') {
      path = String::with_capacity(0);
    }

    let mut file_send_task = FileSendTask {
      response: self.response.to_owned(),
      path,
      options: (&self.options).into(),
    };

    let file_serve_result = file_send_task.compute()?;

    let forward_error = !self.options.fallthrough;
    let response = file_send_task.response.to_owned();
    if forward_error
      && response
        .with_inner(|w_res| Ok(w_res.inner()?.status()))?
        .as_u16()
        >= 400
    {
      return Ok(true);
    }

    if let Some(set_headers_fn) = &self.options.set_headers
      && let Some(file_serve_result) = file_serve_result
    {
      let path = file_serve_result
        .served_path
        .to_str()
        .ok_or(Error::new(
          Status::GenericFailure,
          "Support for non-UTF-8 paths not implemented yet.",
        ))?
        .to_owned();
      let file_stat: FileStat = file_serve_result.file_stat.into();
      // Create Tokio runtime
      let rt = Runtime::new()?;
      //   TODO: Figure out how to get `path` and `file_stat`
      rt.block_on(async {
        set_headers_fn
          .call_async((response, path, file_stat).into())
          .await
      })?;
    }

    Ok(true)
  }

  fn resolve(&mut self, _env: Env, compute_output: Self::Output) -> Result<Self::JsValue> {
    Ok(compute_output)
  }
}
