use std::{str::FromStr, sync::Arc};

use byte_unit::Byte;
use futures::StreamExt;
use http_body_util::{BodyStream, Limited};
use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction},
};
use napi_derive::napi;
use serde_json::Value as JsonValue;

use crate::{request::Request, response::Response, utilities};

type ThreadsafeParseTypeFn =
  ThreadsafeFunction<FnArgs<(Request,)>, bool, FnArgs<(Request,)>, Status, false, false, 0>;

type ThreadsafeVerifyFn = ThreadsafeFunction<
  FnArgs<(Request, Response, Buffer, String)>,
  (),
  FnArgs<(Request, Response, Buffer, String)>,
  Status,
  false,
  false,
  0,
>;

type JsVerifyFn<'a> = Function<'a, FnArgs<(Request, Response, Buffer, String)>, ()>;

#[napi(object)]
pub struct JsJsonOptions<'a> {
  /// Enables or disables handling deflated (compressed) bodies; when disabled,
  /// deflated bodies are rejected.
  ///
  /// Default = true
  pub inflate: Option<bool>,

  /// Controls the maximum request body size. If this is a number, then the
  /// value specifies the number of bytes; if it is a string, the value is
  /// passed to the [bytes](https://docs.rs/bytesize/2.3.1/bytesize/) library
  /// for parsing.
  ///
  /// Default = "100kb"
  pub limit: Option<Either<i64, String>>,

  /// TODO: Add support for reviver option
  ///
  /// Default = none
  pub reviver: Option<Function<'a, FnArgs<(String, Object<'static>, Object<'static>)>>>,

  /// Enables or disables only accepting arrays and objects; when disabled will
  /// accept anything `serde_json::from_slice` accepts.
  ///
  /// Default = true
  pub strict: Option<bool>,

  /// This is used to determine what media type the middleware will parse. This
  /// option can be a string, array of strings, or a function. If not a
  /// function, `type` option is passed directly to the
  /// [mime_guess](https://docs.rs/mime_guess/latest/mime_guess/) library and
  /// this can be an extension name (like `json`), a mime type (like
  /// `application/json`), or a mime type with a wildcard (like `*/*` or
  /// `*/json`).
  ///
  /// If a function, the type option is called as `fn(req)` and the request is
  /// parsed if it return `true`.
  ///
  /// Default = "application/json"
  pub typ: Option<Either3<String, Vec<String>, Function<'a, Request, bool>>>,

  /// This option, if supplied, is called as `verify(req, res, buf, encoding)`,
  /// where `buf` is a `Buffer` of the raw request body and `encoding` is the
  /// encoding of the request. The parsing can be aborted by throwing an error.
  ///
  /// Default = none
  pub verify: Option<JsVerifyFn<'a>>,
}

struct JsonOptions {
  inflate: bool,
  limit: usize,
  // TODO: reviver
  strict: bool,
  typ: Either<Vec<String>, Arc<ThreadsafeParseTypeFn>>,
  verify: Option<Arc<ThreadsafeVerifyFn>>,
}

impl Default for JsonOptions {
  fn default() -> Self {
    Self {
      inflate: false,
      limit: 102_400, // 100kb
      strict: true,
      typ: Either::A(vec!["application/json".to_owned()]),
      verify: None,
    }
  }
}

impl<'a> JsJsonOptions<'a> {
  fn to_json_options(&self) -> Result<JsonOptions> {
    let mut json_options = JsonOptions::default();

    if let Some(inflate) = self.inflate {
      json_options.inflate = inflate;
    }

    if let Some(limit) = &self.limit {
      match limit {
        Either::A(limit) => {
          json_options.limit = *limit as usize;
        }
        Either::B(limit) => {
          let limit = utilities::decimal_to_binary_unit(limit);
          match Byte::from_str(&limit) {
            Ok(limit) => {
              json_options.limit = limit.as_u64() as usize;
            }
            Err(e) => {
              return Err(Error::new(
                Status::InvalidArg,
                format!("Invalid limit value: {e}"),
              ));
            }
          }
        }
      }
    }

    if let Some(strict) = self.strict {
      json_options.strict = strict;
    }

    if let Some(media_type) = &self.typ {
      match media_type {
        Either3::A(media_type) => json_options.typ = Either::A(vec![media_type.to_owned()]),
        Either3::B(media_types) => json_options.typ = Either::A(media_types.to_owned()),
        Either3::C(media_type_fn) => {
          let tsfn = media_type_fn
            .build_threadsafe_function()
            .build_callback(|ctx: ThreadsafeCallContext<FnArgs<(Request,)>>| Ok(ctx.value))?;
          json_options.typ = Either::B(Arc::new(tsfn));
        }
      }
    }

    if let Some(verify_fn) = &self.verify {
      let tsfn = verify_fn.build_threadsafe_function().build_callback(
        |ctx: ThreadsafeCallContext<FnArgs<(Request, Response, Buffer, String)>>| Ok(ctx.value),
      )?;
      json_options.verify = Some(Arc::new(tsfn));
    }

    Ok(json_options)
  }
}

impl JsonOptions {
  async fn should_parse(&self, request: &Request) -> Result<bool> {
    let req_content_type = match request.get(hyper::http::header::CONTENT_TYPE.to_string())? {
      Either::A(val) => val,
      Either::B(_) => return Ok(false),
    };
    match &self.typ {
      Either::A(types) => {
        let types = types.iter().map(|s| s.as_str()).collect::<Vec<_>>();
        Ok(utilities::type_is(&req_content_type, &types).is_some())
      }
      Either::B(should_parse_fn) => {
        should_parse_fn
          .call_async((request.to_owned(),).into())
          .await
      }
    }
  }
}

#[napi]
pub struct JsonMiddleware {
  options: JsonOptions,
}

#[napi]
impl JsonMiddleware {
  #[napi(constructor)]
  pub fn new(options: Option<JsJsonOptions>) -> Result<Self> {
    Ok(JsonMiddleware {
      options: match options {
        Some(options) => options.to_json_options()?,
        None => JsonOptions::default(),
      },
    })
  }

  #[napi]
  pub async fn run(&self, request: &Request, _response: &Response) -> Result<bool> {
    println!("Json Middleware | Called!");

    // determine if request should be parsed
    if !self.options.should_parse(request).await? {
      return Ok(true);
    }

    let hyper_request = request.with_inner_mut(|req| req.take_inner())?;
    let mut body_stream = BodyStream::new(Limited::new(hyper_request, self.options.limit));

    let mut body = Vec::new();
    while let Some(data) = body_stream.next().await {
      let data = data
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?
        .into_data()
        .map_err(|_| Error::new(Status::GenericFailure, "Encountered a non-data frame."))?;
      body.extend_from_slice(&data);
    }

    // skip requests without bodies
    if body.is_empty() {
      return Ok(true);
    }

    if self.options.strict && body[0] != b'{' && body[0] != b'[' {
      return Err(Error::new(
        Status::GenericFailure,
        "Expected body to be a JSON array or object.",
      ));
    }

    let parsed_json: JsonValue = serde_json::from_slice(&body).map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Error parsing JSON body: {e}"),
      )
    })?;

    request.with_inner_mut(|req| {
      req.set_body(Either::B(parsed_json));
      Ok(())
    })?;
    Ok(true)
  }
}
