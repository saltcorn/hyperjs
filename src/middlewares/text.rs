use futures::StreamExt;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::{request::Request, response::Response};

#[napi(object)]
pub struct TextOptions {}

#[napi]
pub struct TextMiddleware {
  options: Option<TextOptions>,
}

#[napi]
impl TextMiddleware {
  #[napi(constructor)]
  pub fn new(options: Option<TextOptions>) -> Self {
    TextMiddleware { options }
  }

  #[napi]
  pub async fn run(&self, request: &Request, _response: &Response) -> Result<bool> {
    println!("Text Middleware | Called!");

    let mut body_stream = request.with_inner(|req| req.body())?;

    let mut body = Vec::new();
    while let Some(data) = body_stream.next().await {
      let data = data.map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
      body.extend_from_slice(&data);
    }

    let req_inner =
      String::from_utf8(body).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    request.with_inner(|req| {
      req.set_body(Either::A(req_inner));
      Ok(())
    })?;
    Ok(true)
  }
}
