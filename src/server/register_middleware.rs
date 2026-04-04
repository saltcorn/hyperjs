use napi::bindgen_prelude::*;
use napi::threadsafe_function::ThreadsafeCallContext;
use std::sync::Arc;

use super::{JsHandlerFn, MiddlewareMeta, Server};
use crate::request::Request;
use crate::response::Response;

impl Server {
  pub(super) fn register_middleware(
    &mut self,
    route: Option<String>,
    handler: JsHandlerFn,
    _env: Env,
  ) -> Result<()> {
    let tsfn = handler
      .build_threadsafe_function()
      .build_callback(|ctx: ThreadsafeCallContext<FnArgs<(Request, Response)>>| Ok(ctx.value))?;
    self.middlewares.push(MiddlewareMeta {
      route,
      handler: Arc::new(tsfn),
      method: None,
    });
    Ok(())
  }
}
