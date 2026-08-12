use napi::threadsafe_function::ThreadsafeCallContext;
use napi::{UnknownRef, bindgen_prelude::*};
use serde_json::Value as JsonValue;
use std::sync::Arc;

use super::{JsHandlerFn, MiddlewareMeta, Server};
use crate::request::Request;
use crate::response::Response;
use crate::server::{
  JsHandlerFnErrorHandler, ThreadsafeMiddlewareErrorHandlerFn, ThreadsafeMiddlewareFn,
};
use crate::utilities;

impl Server {
  pub(super) fn register_middleware(
    &mut self,
    route: Option<String>,
    handler: Either<JsHandlerFn, JsHandlerFnErrorHandler>,
    _env: Env,
  ) -> Result<()> {
    let tsfn = Self::get_threadsafe_middleware_fn(handler)?;

    self.middlewares.push(MiddlewareMeta {
      route,
      handler: Arc::new(tsfn),
      method: None,
    });
    Ok(())
  }

  pub(super) fn get_threadsafe_middleware_fn(
    handler: Either<JsHandlerFn, JsHandlerFnErrorHandler>,
  ) -> napi::Result<Either<ThreadsafeMiddlewareFn, ThreadsafeMiddlewareErrorHandlerFn>> {
    match handler {
      Either::A(normal) => {
        let tsfn = normal.build_threadsafe_function().build_callback(
          |ctx: ThreadsafeCallContext<FnArgs<(Request, Response)>>| Ok(ctx.value),
        )?;

        Ok(Either::A(tsfn))
      }
      Either::B(error_handling) => {
        let tsfn = error_handling.build_threadsafe_function().build_callback(
          |ctx: ThreadsafeCallContext<FnArgs<(JsonValue, Request, Response)>>| {
            let napi_v = utilities::json_to_napi(&ctx.env, ctx.value.data.0)?.create_ref()?;

            Ok(FnArgs::from((napi_v, ctx.value.data.1, ctx.value.data.2)))
          },
        )?;

        Ok(Either::B(tsfn))
      }
    }
  }
}
