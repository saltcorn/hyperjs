use std::collections::HashMap;

use hyper::header::{ACCEPT, CONTENT_TYPE};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Response, WrappedResponse};
use crate::request::Request;

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
  pub fn formut(&self, obj: Object<'static>) -> Result<Self> {
    let mut format_fns = HashMap::new();
    for key in Object::keys(&obj)? {
      let format_fn = obj
        .get::<Function<FnArgs<(Request, Response)>, ()>>(&key)?
        .ok_or(Error::new(
          Status::GenericFailure,
          "Expected format object key to have a value.",
        ))?;
      format_fns.insert(key, format_fn);
    }
    WrappedResponse::formut(format_fns, self.req(), self.clone())?;
    Ok(self.clone())
  }
}

impl WrappedResponse {
  pub fn formut(
    obj: HashMap<String, Function<FnArgs<(Request, Response)>, ()>>,
    req: Request,
    mut res: Response,
  ) -> Result<()> {
    println!("One");
    let keys = obj
      .keys()
      .filter(|key| *key != "default")
      .cloned()
      .collect::<Vec<_>>();

    println!("Two");
    res.vary(ACCEPT.as_str().to_owned())?;

    println!("Three");
    let key = req.accepts(Either::B(keys))?.and_then(|v| match v {
      Either::A(val) => Some(val),
      Either::B(vals) => vals.first().cloned(),
    });

    println!("Client ACCEPT key = {key:?}");

    match key {
      Some(key) => {
        res.set_string(CONTENT_TYPE.to_string(), key.to_owned())?;
        match obj.get(&key) {
          Some(handler_fn) => handler_fn.call((req, res).into()),
          None => Err(Error::new(
            Status::GenericFailure,
            format!("No handler found for {key}"),
          )),
        }
      }
      None => match obj.get("default") {
        Some(handler_fn) => handler_fn.call((req, res).into()),
        None => Err(Error::new(
          Status::InvalidArg,
          "Handler function for 'default' must exist in supplied object.",
        )),
      },
    }
  }
}
