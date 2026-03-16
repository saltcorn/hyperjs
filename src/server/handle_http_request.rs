use std::fmt::Display;
use std::sync::Arc;

use headers_core::HeaderValue;
use hyper::StatusCode;
use hyper::header::{CONTENT_SECURITY_POLICY, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS};
use hyper::{Request as HyperRequest, Response as HyperResponse, body::Incoming as IncomingBody};
use matchit::Router;
use napi::Either;

use super::get_next_id::get_next_id;
use crate::request::{Request, WrappedRequest};
use crate::response::{CrateBody, Response};
use crate::server::MiddlewareMeta;
use crate::utilities::full;

fn log_napi_error(mut error: &napi::Error) -> String {
  let mut error_message = error.to_string();
  while let Some(cause) = error.cause.as_deref() {
    error_message.push_str("<br> &nbsp; &nbsp;");
    error_message.push_str(&cause.to_string());
    error = cause
  }
  error_message
}

fn create_error_500<T: Display>(e: T) -> HyperResponse<CrateBody> {
  let mut response_builder = HyperResponse::builder();
  if let Some(headers) = response_builder.headers_mut() {
    headers.insert(
      CONTENT_SECURITY_POLICY,
      HeaderValue::from_static("default-src 'none'"),
    );
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(
      CONTENT_TYPE,
      HeaderValue::from_static("text/html; charset=utf-8"),
    );
  };
  let page_content = format!(
    r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>Error</title>
</head>
<body>
<pre>Error: {e}</pre>
</body>
</html>
"#
  );
  response_builder
    .status(500)
    .body(full(page_content))
    .unwrap()
}

pub(super) async fn handle_http_request(
  req: HyperRequest<IncomingBody>,
  router: Arc<Router<String>>,
  middlewares: Arc<Vec<MiddlewareMeta>>,
) -> std::result::Result<HyperResponse<CrateBody>, Box<dyn std::error::Error + Sync + Send>> {
  let request_id = get_next_id();
  log::debug!("Generated request_id={request_id}.");

  log::debug!("--- Handling new HTTP request ---");
  let request_method = req.method().to_owned();
  let request_uri = req.uri().to_owned();
  let request_version = req.version();
  log::debug!(
    "Request ID: {request_id} | Method: {:?}, URI: {:?}, Version: {:?}",
    request_method,
    request_uri,
    request_version
  );
  log::debug!("Headers: {:?}", req.headers());

  let body_request: WrappedRequest = req.into();
  let request = Request::from(body_request);
  let response = Response::new(request.clone(), None);

  for middleware in middlewares.as_ref() {
    log::debug!(
      "Looping through middlewares ({}, {}) ...",
      middleware
        .method
        .as_ref()
        .map(|s| s.to_string())
        .unwrap_or_default(),
      middleware.route.as_ref().cloned().unwrap_or_default()
    );
    // if the middleware is associated to a route:
    // 1. assert middleware's route matches request's route, else skip
    //    middleware's execution
    // 2. if request & middleware's routes match, save extracted params
    //    in request
    if let Some(middleware_route) = middleware.route.as_ref() {
      let request_uri_string = request_uri.to_string();
      match router.at(&request_uri_string) {
        Ok(router_match) => {
          let params = router_match.params;
          let request_route = router_match.value;
          match request_route == middleware_route {
            true => {
              // TODO: Avoid setting params if already set e.g if a route has
              // more than one middleware registered for it.
              if let Err(e) = request.with_inner_mut(|w_req| {
                w_req.set_params(params.iter());
                Ok(())
              }) {
                let err_msg = format!("Error setting request parameters: {e}");
                log::debug!("Request ID: {request_id} | {err_msg}.");
                return Ok(
                  HyperResponse::builder()
                    .status(500)
                    .body(full(err_msg))
                    .unwrap(),
                );
              };
            }
            false => continue,
          }
        }
        Err(_) => continue,
      };
    }

    // if the middleware is associated to a particular HTTP method:
    // - Execute it's handler only if the middleware's method matches the
    //   specified method
    if let Some(middleware_method) = middleware.method.as_ref() {
      let request_method = match request.with_inner(|w_req| Ok(w_req.inner()?.method().to_owned()))
      {
        Ok(method) => method,
        Err(e) => {
          let err_msg = format!("Error getting request's method: {e}");
          log::debug!("Request ID: {request_id} | {err_msg}.");
          return Ok(
            HyperResponse::builder()
              .status(500)
              .body(full(err_msg))
              .unwrap(),
          );
        }
      };
      if request_method != middleware_method {
        continue;
      }
    }

    log::debug!("Request ID: {request_id} | Calling JS middleware.");
    let middleware_response = match middleware
      .handler
      .call_async((request.clone(), response.clone()).into())
      .await
    {
      Ok(response) => response,
      Err(e) => {
        log::debug!("Request ID: {request_id} | JS middleware invocation failed.");
        let err_msg = format!("Failed to invoke middleware: {e}.");
        return Ok(
          HyperResponse::builder()
            .status(500)
            .body(full(err_msg))
            .unwrap(),
        );
      }
    };

    log::debug!("Request ID: {request_id} | JS middleware called successfully.");

    log::debug!("Request ID: {request_id} | Waiting for JS middleware (30s timeout)");

    let middleware_execution_result = match middleware_response {
      Either::A(continue_flag) => continue_flag,
      Either::B(promise) => {
        match tokio::time::timeout(std::time::Duration::from_secs(30), promise).await {
          Ok(Ok(continue_flag)) => continue_flag,
          Ok(Err(e)) => {
            log::debug!("Request ID: {request_id} | Middleware execution failed.",);
            log::debug!("Request ID: {request_id} | {e}");
            return Ok(create_error_500(log_napi_error(&e)));
          }
          Err(e) => {
            log::debug!("Request ID: {request_id} | JS middleware timeout.");
            log::debug!("Request ID: {request_id} | {e}");

            return Ok(
              HyperResponse::builder()
                .status(504)
                .body(full("Middleware timeout"))
                .unwrap(),
            );
          }
        }
      }
    };

    log::debug!("Middleware execution result: {middleware_execution_result:?}");

    match middleware_execution_result {
      Either::A(should_continue) => match should_continue {
        true => {}
        false => break,
      },
      Either::B(_) => break,
    }
  }

  log::debug!("Request ID: {request_id} | Received response from JS");

  let resp = response;
  let status_code =
    match resp.with_inner(|response| napi::Result::<StatusCode>::Ok(response.inner()?.status())) {
      Ok(status_code) => status_code,
      Err(e) => {
        log::debug!("Request ID: {request_id} | Inner response acquisition failed.");
        let err_msg = format!("Failed to acquire the wrapped response: {e}.");
        return Ok(
          HyperResponse::builder()
            .status(500)
            .body(full(err_msg))
            .unwrap(),
        );
      }
    };
  log::debug!(
    "Request ID: {request_id} | Responding with status={}.",
    status_code
  );

  let resp = resp.with_inner(|r| r.take()).unwrap();

  Ok(resp)
}
