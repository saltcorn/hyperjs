use crate::request::Request;
use napi_derive::napi;

// Context passed to JavaScript handler
#[napi]
pub struct RequestContext {
  request: Request,
  request_id: u32,
}

impl RequestContext {
  pub fn new(request: Request, request_id: u32) -> Self {
    Self {
      request,
      request_id,
    }
  }
}

#[napi]
impl RequestContext {
  #[napi(getter)]
  pub fn request(&self) -> Request {
    self.request.clone()
  }

  #[napi(getter)]
  pub fn request_id(&self) -> u32 {
    self.request_id
  }
}
