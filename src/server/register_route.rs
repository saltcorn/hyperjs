use hyper::Method as LibMethod;
use matchit::InsertError;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeCallContext;
use std::sync::Arc;

use super::{JsHandlerFn, MiddlewareMeta, Server};
use crate::request::Request;
use crate::response::Response;

impl Server {
  pub(super) fn register_route(
    &mut self,
    route: String,
    handler: JsHandlerFn,
    method: LibMethod,
  ) -> Result<()> {
    let tsfn = handler
      .build_threadsafe_function()
      .build_callback(|ctx: ThreadsafeCallContext<FnArgs<(Request, Response)>>| Ok(ctx.value))?;
    if let Err(e) = self.router.insert(route.to_owned(), route.to_owned()) {
      match e {
        InsertError::Conflict { .. } => {}
        _ => return Err(Error::new(Status::GenericFailure, e.to_string())),
      }
    }
    self.middlewares.push(MiddlewareMeta {
      route: Some(route),
      handler: Arc::new(tsfn),
      method: Some(method),
    });
    Ok(())
  }
}
