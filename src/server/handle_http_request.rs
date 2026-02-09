use std::sync::Arc;

use hyper::StatusCode;
use hyper::{
  Error as LibError, Request as HyperRequest, Response as HyperResponse,
  body::Incoming as IncomingBody,
};
use napi::Either;

use super::get_next_id::get_next_id;
use crate::request::{Request, WrappedRequest};
use crate::response::{CrateBody, Response};
use crate::server::{MiddlewaresRouter, RoutersMap, ThreadsafeMiddlewareFn};
use crate::utilities::{body_from_status_code, full};

async fn run_app_wide_middlewares(
  request_id: u32,
  request: &Request,
  response: &Response,
  app_wide_middleware: Vec<Arc<ThreadsafeMiddlewareFn>>,
) -> std::result::Result<Option<HyperResponse<CrateBody>>, LibError> {
  for middleware in app_wide_middleware {
    println!("Request ID: {request_id} | Calling JS middleware.");
    let middleware_response = match middleware
      .call_async((request.clone(), response.clone()).into())
      .await
    {
      Ok(response) => response,
      Err(e) => {
        println!("Request ID: {request_id} | JS middleware invocation failed.");
        let err_msg = format!("Failed to invoke middleware: {e}.");
        return Ok(Some(
          HyperResponse::builder()
            .status(500)
            .body(full(err_msg))
            .unwrap(),
        ));
      }
    };

    println!("Request ID: {request_id} | JS middleware called successfully.");

    println!("Request ID: {request_id} | Waiting for JS middleware (30s timeout)");

    let should_continue = match middleware_response {
      Either::A(continue_flag) => continue_flag,
      Either::B(promise) => {
        match tokio::time::timeout(std::time::Duration::from_secs(30), promise).await {
          Ok(Ok(continue_flag)) => continue_flag,
          Ok(Err(e)) => {
            println!("Request ID: {request_id} | Middleware execution failed.",);
            println!("Request ID: {request_id} | {e}");

            return Ok(Some(
              HyperResponse::builder()
                .status(500)
                .body(full("Middleware failed to terminate"))
                .unwrap(),
            ));
          }
          Err(e) => {
            println!("Request ID: {request_id} | JS middleware timeout.");
            println!("Request ID: {request_id} | {e}");

            return Ok(Some(
              HyperResponse::builder()
                .status(504)
                .body(full("Middleware timeout"))
                .unwrap(),
            ));
          }
        }
      }
    };

    if !should_continue {
      let resp = response.with_inner(|r| r.take()).unwrap();
      return Ok(Some(resp));
    }
  }

  Ok(None)
}

pub(super) async fn handle_http_request(
  req: HyperRequest<IncomingBody>,
  routers_map: RoutersMap,
  middlewares_router: MiddlewaresRouter,
  app_wide_middleware: Vec<Arc<ThreadsafeMiddlewareFn>>,
) -> std::result::Result<HyperResponse<CrateBody>, LibError> {
  let request_id = get_next_id();
  println!("Generated request_id={request_id}.");

  println!("--- Handling new HTTP request ---");
  let request_method = req.method().to_owned();
  let request_uri = req.uri().to_owned();
  let request_version = req.version();
  println!(
    "Request ID: {request_id} | Method: {:?}, URI: {:?}, Version: {:?}",
    request_method, request_uri, request_version
  );
  println!("Headers: {:?}", req.headers());

  let body_request: WrappedRequest = req.into();
  let request = Request::from(body_request);
  let response = Response::new(request.clone(), None);

  // run app-wide middlewares
  if let Some(res) =
    run_app_wide_middlewares(request_id, &request, &response, app_wide_middleware).await?
  {
    return Ok(res);
  }

  let request_uri_string = request_uri.to_string();
  let routers_map = match routers_map.read() {
    Ok(router) => router.clone(),
    Err(e) => {
      println!("Request ID: {request_id} | Unable to obtain read access to router.");
      let err_msg = format!("Failed to obtain read access to router: {e}.");
      return Ok(
        HyperResponse::builder()
          .status(500)
          .body(full(err_msg))
          .unwrap(),
      );
    }
  };
  let router = match routers_map.get(&request_method) {
    Some(router) => router,
    None => {
      println!("Request ID: {request_id} | Not found.");
      let status_code = StatusCode::NOT_FOUND;
      let body = body_from_status_code(status_code);
      return Ok(HyperResponse::builder().status(404).body(body).unwrap());
    }
  };

  let (route_meta, params) = match router.at(&request_uri_string) {
    Ok(route_match) => (route_match.value.to_owned(), route_match.params),
    Err(_) => {
      println!("Request ID: {request_id} | Not found.");
      let status_code = StatusCode::NOT_FOUND;
      let body = body_from_status_code(status_code);
      return Ok(HyperResponse::builder().status(404).body(body).unwrap());
    }
  };

  if let Err(e) = request.with_inner_mut(|w_req| {
    w_req.set_params(params.iter());
    Ok(())
  }) {
    let err_msg = format!("Error setting request parameters: {e}");
    println!("Request ID: {request_id} | {err_msg}.");
    return Ok(
      HyperResponse::builder()
        .status(500)
        .body(full(err_msg))
        .unwrap(),
    );
  };

  let middlewares_meta = match middlewares_router.read() {
    Ok(middlewares_meta) => middlewares_meta.to_owned(),
    Err(e) => {
      let err_msg = format!("Error obtaining read lock on middlewares meta: {e}");
      println!("Request ID: {request_id} | {err_msg}.");
      return Ok(
        HyperResponse::builder()
          .status(500)
          .body(full(err_msg))
          .unwrap(),
      );
    }
  };

  // run route-specific middlewares
  if let Ok(route_match_data) = middlewares_meta.at(&request_uri_string) {
    let next_called = route_match_data.value.next_called.clone();
    let middlewares = route_match_data.value.middlewares.clone();

    for middleware in middlewares {
      println!("Request ID: {request_id} | Calling JS middleware.");
      let middleware_response = match middleware
        .call_async((request.clone(), response.clone()).into())
        .await
      {
        Ok(response) => response,
        Err(e) => {
          println!("Request ID: {request_id} | JS middleware invocation failed.");
          let err_msg = format!("Failed to invoke middleware: {e}.");
          return Ok(
            HyperResponse::builder()
              .status(500)
              .body(full(err_msg))
              .unwrap(),
          );
        }
      };

      println!("Request ID: {request_id} | JS middleware called successfully.");

      println!("Request ID: {request_id} | Waiting for JS middleware (30s timeout)");

      let should_continue = match middleware_response {
        Either::A(continue_flag) => continue_flag,
        Either::B(promise) => {
          match tokio::time::timeout(std::time::Duration::from_secs(30), promise).await {
            Ok(Ok(continue_flag)) => continue_flag,
            Ok(Err(e)) => {
              println!("Request ID: {request_id} | Middleware execution failed.",);
              println!("Request ID: {request_id} | {e}");

              return Ok(
                HyperResponse::builder()
                  .status(500)
                  .body(full("Middleware failed to terminate"))
                  .unwrap(),
              );
            }
            Err(e) => {
              println!("Request ID: {request_id} | JS middleware timeout.");
              println!("Request ID: {request_id} | {e}");

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

      // Update next_called flag based on middleware return value
      match next_called.lock() {
        Ok(mut next_called) => {
          *next_called = should_continue;
          if !should_continue {
            println!("Request ID: {request_id} | Middleware returned false, stopping chain.");
            break;
          }
        }
        Err(e) => {
          let error_msg = "Failed to acquire lock on middleware 'next_called' status.";
          println!("Request ID: {request_id} | {error_msg}",);
          println!("Request ID: {request_id} | {e}");

          return Ok(
            HyperResponse::builder()
              .status(500)
              .body(full(error_msg))
              .unwrap(),
          );
        }
      }
    }
  }

  println!("Request ID: {request_id} | Calling JS handler.");
  let handler_response = match route_meta
    .handler
    .call_async((request, response.clone()).into())
    .await
  {
    Ok(response) => response,
    Err(e) => {
      println!("Request ID: {request_id} | JS handler invocation failed.");
      let err_msg = format!("Failed to invoke handler: {e}.");
      return Ok(
        HyperResponse::builder()
          .status(500)
          .body(full(err_msg))
          .unwrap(),
      );
    }
  };

  println!("Request ID: {request_id} | JS handler called successfully.");

  println!("Request ID: {request_id} | Waiting for JS response (30s timeout)");

  match handler_response {
    Either::A(_) => {}
    Either::B(promise) => {
      match tokio::time::timeout(std::time::Duration::from_secs(30), promise).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => {
          println!("Request ID: {request_id} | Response channel closed without response.",);
          println!("Request ID: {request_id} | {e}");

          return Ok(
            HyperResponse::builder()
              .status(500)
              .body(full("Handler failed to respond"))
              .unwrap(),
          );
        }
        Err(e) => {
          println!("Request ID: {request_id} | JS handler timeout.");
          println!("Request ID: {request_id} | {e}");

          return Ok(
            HyperResponse::builder()
              .status(504)
              .body(full("Handler timeout"))
              .unwrap(),
          );
        }
      }
    }
  }

  println!("Request ID: {request_id} | Received response from JS");

  let resp = response;
  let status_code =
    match resp.with_inner(|response| napi::Result::<StatusCode>::Ok(response.inner()?.status())) {
      Ok(status_code) => status_code,
      Err(e) => {
        println!("Request ID: {request_id} | Inner response acquisition failed.");
        let err_msg = format!("Failed to acquire the wrapped response: {e}.");
        return Ok(
          HyperResponse::builder()
            .status(500)
            .body(full(err_msg))
            .unwrap(),
        );
      }
    };
  println!(
    "Request ID: {request_id} | Responding with status={}.",
    status_code
  );

  let resp = resp.with_inner(|r| r.take()).unwrap();

  Ok(resp)
}
