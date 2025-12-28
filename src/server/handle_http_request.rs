use bytes::Bytes;
use http_body_util::{Either, Empty, Full};
use hyper::{body::Incoming as IncomingBody, Request as HyperRequest, Response as HyperResponse};

use super::get_next_id::get_next_id;
use crate::request::interface::RequestInterface;
use crate::request::Request;
use crate::server::RoutersMap;

type HandlerReturn = Either<Full<Bytes>, Empty<Bytes>>;

pub(super) async fn handle_http_request(
  req: HyperRequest<IncomingBody>,
  routers_map: RoutersMap,
) -> std::result::Result<HyperResponse<HandlerReturn>, hyper::Error> {
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
          .body(Either::Left(Full::new(Bytes::from(err_msg))))
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
          .body(Either::Right(Empty::new()))
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
          .body(Either::Right(Empty::new()))
          .unwrap(),
      );
    }
  };

  let body_request: Box<dyn RequestInterface> = Box::new(req);
  let our_request = Request::from(body_request);

  println!("Request ID: {request_id} | Calling JS handler.");
  let handler_promise = match handler_fn.call_async(our_request).await {
    Ok(promise) => promise,
    Err(e) => {
      println!("Request ID: {request_id} | JS handler invocation failed.");
      let err_msg = format!("Failed to invoke handler: {e}.");
      return Ok(
        HyperResponse::builder()
          .status(500)
          .body(Either::Left(Full::new(Bytes::from(err_msg))))
          .unwrap(),
      );
    }
  };

  println!("Request ID: {request_id} | JS handler called successfully.");

  println!("Request ID: {request_id} | Waiting for JS response (30s timeout)");

  match tokio::time::timeout(std::time::Duration::from_secs(30), handler_promise).await {
    Ok(Ok(response)) => {
      println!("Request ID: {request_id} | Received response from JS");

      let resp = response;
      let status_code: hyper::http::StatusCode = (&resp.status()).into();
      println!(
        "Request ID: {request_id} | Responding with status={}.",
        status_code
      );

      Ok(resp.owned_inner())
    }
    Ok(Err(_)) => {
      println!("Request ID: {request_id} | Response channel closed without response.",);

      Ok(
        HyperResponse::builder()
          .status(500)
          .body(Either::Left(Full::new(Bytes::from(
            "Handler failed to respond",
          ))))
          .unwrap(),
      )
    }
    Err(_) => {
      println!("Request ID: {request_id} | JS handler timeout.");

      Ok(
        HyperResponse::builder()
          .status(504)
          .body(Either::Left(Full::new(Bytes::from("Handler timeout"))))
          .unwrap(),
      )
    }
  }
}
