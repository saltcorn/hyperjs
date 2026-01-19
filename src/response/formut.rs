use std::collections::HashMap;

use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction},
};
use napi_derive::napi;

use super::{Response, WrappedResponse};
use crate::request::Request;

type ThreadsafeFormatFn = ThreadsafeFunction<
  FnArgs<(Request, Response)>,
  (),
  FnArgs<(Request, Response)>,
  Status,
  false,
  false,
  0,
>;

#[napi]
impl Response {
  /// Returns the HTTP response header specified by `field`. The match is case-insensitive.
  ///
  /// ```javascript
  /// res.get('Content-Type')
  /// // => "text/plain"
  /// ```
  #[napi(js_name = "format")]
  pub fn formut(&mut self, obj: Object) -> Result<Either<String, Buffer>> {
    let mut format_fns = HashMap::new();
    for key in Object::keys(&obj)? {
      let format_fn = obj
        .get::<Function<FnArgs<(Request, Response)>, ()>>(&key)?
        .ok_or(Error::new(
          Status::GenericFailure,
          "Expected format object key to have a value.",
        ))?;
      let tsfn = format_fn
        .build_threadsafe_function()
        .build_callback(|ctx: ThreadsafeCallContext<FnArgs<(Request, Response)>>| Ok(ctx.value))?;
      format_fns.insert(key, tsfn);
    }
    self.with_inner(|response| response.formut(format_fns))
  }
}

impl WrappedResponse {
  pub fn formut(
    &mut self,
    obj: HashMap<String, ThreadsafeFormatFn>,
  ) -> Result<Either<String, Buffer>> {
    let header_values = self
      .inner()?
      .headers()
      .get_all(&field)
      .iter()
      .map(|value| match value.to_str() {
        Ok(value) => Either::A(value),
        Err(_) => Either::B(value.as_bytes()),
      })
      .collect::<Vec<_>>();

    match header_values.iter().any(|v| match v {
      Either::A(_) => false,
      Either::B(_) => true,
    }) {
      true => {
        let byte_values = header_values
          .iter()
          .map(|v| match v {
            Either::A(a) => a.as_bytes(),
            Either::B(b) => b,
          })
          .collect::<Vec<_>>()
          .join(&b", "[..]);
        Ok(Either::B(byte_values.into()))
      }
      false => {
        let str_values = header_values
          .iter()
          .filter_map(|v| match v {
            Either::A(a) => Some(*a),
            Either::B(_) => None,
          })
          .collect::<Vec<_>>()
          .join(", ");
        Ok(Either::A(str_values))
      }
    }
  }
}
