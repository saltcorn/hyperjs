use std::{collections::HashMap, sync::Arc};

use hyper::header::{ACCEPT, CONTENT_TYPE};
use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction},
};
use napi_derive::napi;
use tokio::runtime::Runtime;

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

pub struct FormatTask {
  format_fn: Arc<ThreadsafeFormatFn>,
  req: Request,
  res: Response,
}

#[napi]
impl Task for FormatTask {
  type Output = ();
  type JsValue = Response;

  fn compute(&mut self) -> Result<Self::Output> {
    let rt = Runtime::new().map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    rt.block_on(
      self
        .format_fn
        .call_async((self.req.to_owned(), self.res.to_owned()).into()),
    )
    .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    Ok(())
  }
  fn resolve(&mut self, _: Env, _output: ()) -> Result<Self::JsValue> {
    Ok(self.res.to_owned())
  }
}

#[napi]
impl Response {
  /// Performs content-negotiation on the Accept HTTP header on the request
  /// object, when present. It uses `req.accepts()` to select a handler for the
  /// request, based on the acceptable types ordered by their quality values.
  /// If the header is not specified, the first callback is invoked. When no
  /// match is found, the server responds with 406 "Not Acceptable", or invokes
  /// the default callback.
  ///
  /// The `Content-Type` response header is set when a callback is selected.
  /// However, you may alter this within the callback using methods such as
  /// `res.set()` or `res.type()`.
  ///
  /// The following example would respond with `{ "message": "hey" }` when the
  /// Accept header field is set to "application/json" or "*&#8203;/json" (however, if
  /// it is "*&#8203;/*", then the response will be "hey").
  ///
  ///```javascript
  /// res.format({
  ///   'text/plain' () {
  ///     res.send('hey')
  ///   },
  ///
  ///   'text/html' () {
  ///     res.send('<p>hey</p>')
  ///   },
  ///
  ///   'application/json' () {
  ///     res.send({ message: 'hey' })
  ///   },
  ///
  ///   default () {
  ///     // log the request and respond with 406
  ///     res.status(406).send('Not Acceptable')
  ///   }
  /// })
  /// ```
  ///
  /// In addition to canonicalized MIME types, you may also use extension names
  /// mapped to these types for a slightly less verbose implementation:
  ///
  /// ```javascript
  /// res.format({
  ///   text () {
  ///     res.send('hey')
  ///   },
  ///
  ///   html () {
  ///     res.send('<p>hey</p>')
  ///   },
  ///
  ///   json () {
  ///     res.send({ message: 'hey' })
  ///   }
  /// })
  /// ```
  #[napi(js_name = "format")]
  pub fn formut(&self, obj: Object<'static>) -> Result<AsyncTask<FormatTask>> {
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
      format_fns.insert(key, Arc::new(tsfn));
    }
    WrappedResponse::formut(format_fns, self.req(), self.clone())
  }
}

impl WrappedResponse {
  pub fn formut(
    obj: HashMap<String, Arc<ThreadsafeFormatFn>>,
    req: Request,
    mut res: Response,
  ) -> Result<AsyncTask<FormatTask>> {
    let keys = obj
      .keys()
      .filter(|key| *key != "default")
      .cloned()
      .collect::<Vec<_>>();

    res.vary(ACCEPT.as_str().to_owned())?;

    let key = req.accepts(Either::B(keys))?.and_then(|v| match v {
      Either::A(val) => Some(val),
      Either::B(vals) => vals.first().cloned(),
    });

    println!("Client ACCEPT key = {key:?}");

    match key {
      Some(key) => {
        res.set_string(CONTENT_TYPE.to_string(), key.to_owned())?;
        match obj.get(&key).cloned() {
          Some(handler_fn) => Ok(AsyncTask::new(FormatTask {
            format_fn: handler_fn,
            req,
            res,
          })),
          None => Err(Error::new(
            Status::GenericFailure,
            format!("No handler found for {key}"),
          )),
        }
      }
      None => match obj.get("default").cloned() {
        Some(handler_fn) => Ok(AsyncTask::new(FormatTask {
          format_fn: handler_fn,
          req,
          res,
        })),
        None => Err(Error::new(
          Status::InvalidArg,
          "Handler function for 'default' must exist in supplied object.",
        )),
      },
    }
  }
}
