use crate::response::Response;
use napi::{Error, Result, Status};
use napi_derive::napi;

use super::PENDING_REQUESTS;

/// Complete the handling of a request by sending back a response
#[napi]
pub fn complete_request(request_id: u32, response: &Response) -> Result<()> {
  println!("complete_request called for request_id={}", request_id);

  let response = response.clone();
  let mut pending = PENDING_REQUESTS.lock().unwrap();

  if let Some(sender) = pending.remove(&request_id) {
    println!("Sending response for request_id={}", request_id);

    sender.send(response).map_err(|_| {
      Error::new(
        Status::GenericFailure,
        "Failed to send response - receiver dropped",
      )
    })?;

    println!(
      "Response delivered successfully for request_id={}",
      request_id
    );
    Ok(())
  } else {
    println!(
      "ERROR: No pending request found for request_id={}",
      request_id
    );

    Err(Error::new(
      Status::InvalidArg,
      format!("No pending request with id {}", request_id),
    ))
  }
}
