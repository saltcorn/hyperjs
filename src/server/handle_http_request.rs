use bytes::Bytes;
use http_body_util::Full;
use hyper::{body::Incoming as IncomingBody, Request as HyperRequest, Response as HyperResponse};

use super::get_next_id::get_next_id;
use crate::body::SupportedBodies;
use crate::server::RoutersMap;
use crate::{body::Body, request::Request};

pub(super) async fn handle_http_request(
  req: HyperRequest<IncomingBody>,
  routers_map: RoutersMap,
) -> std::result::Result<HyperResponse<Full<Bytes>>, hyper::Error> {
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
          .body(Full::new(Bytes::from(err_msg)))
          .unwrap(),
      );
    }
  };
  let router = match routers_map.get(request_method) {
    Some(router) => router,
    None => {
      println!("Request ID: {request_id} | Not found.");
      return Ok(
        HyperResponse::builder()
          .status(404)
          .body(Full::new(Bytes::from("")))
          .unwrap(),
      );
    }
  };

  let handler_fn = match router.at(&request_uri_string) {
    Ok(route_match) => route_match.value.to_owned(),
    Err(_) => {
      println!("Request ID: {request_id} | Not found.");
      return Ok(
        HyperResponse::builder()
          .status(404)
          .body(Full::new(Bytes::from("")))
          .unwrap(),
      );
    }
  };

  let body_request: Body = SupportedBodies::Empty.into();
  let our_request = Request::builder()
    .uri(req.uri().to_string())
    .and_then(|mut builder| builder.body(&body_request))
    .unwrap();

  println!("Request ID: {request_id} | Calling JS handler.");
  let handler_promise = match handler_fn.call_async(our_request).await {
    Ok(promise) => promise,
    Err(e) => {
      println!("Request ID: {request_id} | JS handler invocation failed.");
      let err_msg = format!("Failed to invoke handler: {e}.");
      return Ok(
        HyperResponse::builder()
          .status(500)
          .body(Full::new(Bytes::from(err_msg)))
          .unwrap(),
      );
    }
  };

  println!("Request ID: {request_id} | JS handler called successfully.");

  println!("Request ID: {request_id} | Waiting for JS response (30s timeout)");

  match tokio::time::timeout(std::time::Duration::from_secs(30), handler_promise).await {
    Ok(Ok(response)) => {
      println!("Request ID: {request_id} | Received response from JS");

      let body_str = match response.body().inner() {
        crate::body::SupportedBodies::Empty => {
          println!("Request ID: {request_id} | Response body empty");
          String::new()
        }
        crate::body::SupportedBodies::String(s) => {
          println!(
            "Request ID: {request_id} | Response body length={}.",
            s.len()
          );
          s.clone()
        }
      };

      let resp = response;
      let status_code: hyper::http::StatusCode = (&resp.status()).into();
      println!(
        "Request ID: {request_id} | Responding with status={}.",
        status_code
      );

      Ok(
        HyperResponse::builder()
          .status(status_code)
          .body(Full::new(Bytes::from(body_str)))
          .unwrap(),
      )
    }
    Ok(Err(_)) => {
      println!("Request ID: {request_id} | Response channel closed without response.",);

      Ok(
        HyperResponse::builder()
          .status(500)
          .body(Full::new(Bytes::from("Handler failed to respond")))
          .unwrap(),
      )
    }
    Err(_) => {
      println!("Request ID: {request_id} | JS handler timeout.");

      Ok(
        HyperResponse::builder()
          .status(504)
          .body(Full::new(Bytes::from("Handler timeout")))
          .unwrap(),
      )
    }
  }
}
