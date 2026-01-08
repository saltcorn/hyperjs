use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::StatusCode;
use hyper::{
  Error as LibError, Request as HyperRequest, Response as HyperResponse,
  body::Incoming as IncomingBody,
};
use napi::Either;

use super::get_next_id::get_next_id;
use crate::request::{Request, WrappedRequest};
use crate::response::{Response, WrappedResponse};
use crate::server::RoutersMap;
use crate::utilities::{body_from_status_code, full};

type HandlerReturn = BoxBody<Bytes, LibError>;

pub(super) async fn handle_http_request(
  req: HyperRequest<IncomingBody>,
  routers_map: RoutersMap,
) -> std::result::Result<HyperResponse<HandlerReturn>, LibError> {
  let request_id = get_next_id();
  println!("Generated request_id={request_id}.");

  println!("--- Handling new HTTP request ---");
  let request_method = req.method();
  let request_uri = req.uri();
  let request_version = req.version();
  println!(
    "Request ID: {request_id} | Method: {:?}, URI: {:?}, Version: {:?}",
    request_method, request_uri, request_version
  );
  println!("Headers: {:?}", req.headers());

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
  let router = match routers_map.get(request_method) {
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

  let mut body_request: WrappedRequest = req.into();
  body_request.set_params(params.iter());
  let request = Request::from(body_request);

  let response: Response = WrappedResponse::default().into();

  let middlewares_meta = match route_meta.middlewares_meta.read() {
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

  if let Some(middlewares_meta) = middlewares_meta {
    let middlewares = match middlewares_meta.middlewares.read() {
      Ok(middlewares) => middlewares.to_owned(),
      Err(e) => {
        let err_msg = format!("Error obtaining write lock on middlewares lists: {e}");
        println!("Request ID: {request_id} | {err_msg}.");
        return Ok(
          HyperResponse::builder()
            .status(500)
            .body(full(err_msg))
            .unwrap(),
        );
      }
    };

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
      match middlewares_meta.next_called.lock() {
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
