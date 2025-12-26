use bytes::Bytes;
use http_body_util::Full;
use hyper::{body::Incoming as IncomingBody, Request as HyperRequest, Response as HyperResponse};
use napi::threadsafe_function::ThreadsafeFunctionCallMode;
use tokio::sync::oneshot;

use super::{
  get_next_id::get_next_id, request_context::RequestContext, AppThreadsafeFunction,
  PENDING_REQUESTS,
};
use crate::body::SupportedBodies;
use crate::{body::Body, request::Request};

pub(super) async fn handle_http_request(
  req: HyperRequest<IncomingBody>,
  handler_fn: AppThreadsafeFunction,
) -> std::result::Result<HyperResponse<Full<Bytes>>, hyper::Error> {
  println!("--- Handling new HTTP request ---");
  println!(
    "Method: {:?}, URI: {:?}, Version: {:?}",
    req.method(),
    req.uri(),
    req.version()
  );
  println!("Headers: {:?}", req.headers());

  let body_request: Body = SupportedBodies::Empty.into();
  let our_request = Request::builder()
    .uri(req.uri().to_string())
    .and_then(|mut builder| builder.body(&body_request))
    .unwrap();

  let request_id = get_next_id();
  println!("Generated request_id={}", request_id);

  let (tx, rx) = oneshot::channel();

  {
    let mut pending = PENDING_REQUESTS.lock().unwrap();
    pending.insert(request_id, tx);
    println!(
      "Stored pending request_id={}, total_pending={}",
      request_id,
      pending.len()
    );
  }

  let ctx = RequestContext::new(our_request, request_id);

  println!("Calling JS handler for request_id={}", request_id);
  let status = handler_fn.call(ctx, ThreadsafeFunctionCallMode::NonBlocking);
  println!(
    "JS handler call status={:?} for request_id={}",
    status, request_id
  );

  if status != napi::Status::Ok {
    println!("JS handler invocation failed for request_id={}", request_id);
    PENDING_REQUESTS.lock().unwrap().remove(&request_id);

    return Ok(
      HyperResponse::builder()
        .status(500)
        .body(Full::new(Bytes::from("Failed to invoke handler")))
        .unwrap(),
    );
  }

  println!(
    "Waiting for JS response (30s timeout) request_id={}",
    request_id
  );

  match tokio::time::timeout(std::time::Duration::from_secs(30), rx).await {
    Ok(Ok(response)) => {
      println!("Received response from JS for request_id={}", request_id);

      let body_str = match response.body().inner() {
        crate::body::SupportedBodies::Empty => {
          println!("Response body empty");
          String::new()
        }
        crate::body::SupportedBodies::String(s) => {
          println!("Response body length={}", s.len());
          s.clone()
        }
      };

      let mut resp = response;
      let status_code: hyper::http::StatusCode = (&resp.status()).into();
      println!(
        "Responding with status={} for request_id={}",
        status_code, request_id
      );

      Ok(
        HyperResponse::builder()
          .status(status_code)
          .body(Full::new(Bytes::from(body_str)))
          .unwrap(),
      )
    }
    Ok(Err(_)) => {
      println!(
        "Response channel closed without response for request_id={}",
        request_id
      );
      PENDING_REQUESTS.lock().unwrap().remove(&request_id);

      Ok(
        HyperResponse::builder()
          .status(500)
          .body(Full::new(Bytes::from("Handler failed to respond")))
          .unwrap(),
      )
    }
    Err(_) => {
      println!("JS handler timeout for request_id={}", request_id);
      PENDING_REQUESTS.lock().unwrap().remove(&request_id);

      Ok(
        HyperResponse::builder()
          .status(504)
          .body(Full::new(Bytes::from("Handler timeout")))
          .unwrap(),
      )
    }
  }
}
