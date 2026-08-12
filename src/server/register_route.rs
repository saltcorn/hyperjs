use hyper::Method as LibMethod;
use matchit::InsertError;
use napi::bindgen_prelude::*;
use std::sync::Arc;

use super::{JsHandlerFn, MiddlewareMeta, Server};
use crate::server::JsHandlerFnErrorHandler;

impl Server {
  pub(super) fn register_route(
    &mut self,
    route: String,
    handler: Either<JsHandlerFn, JsHandlerFnErrorHandler>,
    method: LibMethod,
  ) -> Result<()> {
    let tsfn = Self::get_threadsafe_middleware_fn(handler)?;

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
