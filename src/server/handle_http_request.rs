use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::StatusCode;
use hyper::{
  Error as LibError, Request as HyperRequest, Response as HyperResponse,
  body::Incoming as IncomingBody,
};
use napi::Either;

use super::get_next_id::get_next_id;
use crate::request::Request;
use crate::request::interface::RequestInterface;
use crate::response::Response;
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

  let (handler_fn, params) = match router.at(&request_uri_string) {
    Ok(route_match) => (route_match.value.to_owned(), route_match.params),
    Err(_) => {
      println!("Request ID: {request_id} | Not found.");
      let status_code = StatusCode::NOT_FOUND;
      let body = body_from_status_code(status_code);
      return Ok(HyperResponse::builder().status(404).body(body).unwrap());
    }
  };

  let body_request: Box<dyn RequestInterface> = Box::new(req);
  let mut our_request = Request::from(body_request);
  our_request.set_params(params.iter());

  println!("Request ID: {request_id} | Calling JS handler.");
  let handler_response = match handler_fn
    .call_async((our_request, Response::default()).into())
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

  let response = match handler_response {
    Either::A(response_object) => response_object,
    Either::B(promise) => {
      match tokio::time::timeout(std::time::Duration::from_secs(30), promise).await {
        Ok(Ok(response)) => response,
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
  };

  println!("Request ID: {request_id} | Received response from JS");

  let resp = response;
  let status_code = match resp.inner() {
    Ok(response) => response.status(),
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

  Ok(resp.take().unwrap())
}
