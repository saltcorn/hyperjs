use std::{str::FromStr, sync::Arc};

use byte_unit::Byte;
use futures::StreamExt;
use http_body_util::{BodyStream, Limited, combinators::BoxBody};
use hyper::Request as HyperRequest;
use napi::{
  bindgen_prelude::*,
  threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction},
};
use napi_derive::napi;

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
pub struct JsRawOptions<'a> {
  /// Enables or disables handling deflated (compressed) bodies; when disabled,
  /// deflated bodies are rejected.
  ///
  /// Default = true
  pub inflate: Option<bool>,

  /// Controls the maximum request body size. If this is a number, then the
  /// value specifies the number of bytes; if it is a string, the value is
  /// passed to the [bytes](https://docs.rs/byte-unit/latest/byte_unit/)
  /// library for parsing.
  ///
  /// Default = "100kb"
  pub limit: Option<Either<i64, String>>,

  /// This is used to determine what media type the middleware will parse. This
  /// option can be a string, array of strings, or a function. If not a
  /// function, `type` option is passed directly to the
  /// [mime_guess](https://docs.rs/mime_guess/latest/mime_guess/) library and
  /// this can be an extension name (like `bin`), a mime type (like
  /// `application/octet-stream`), or a mime type with a wildcard (like `*/*`
  /// or `application/*`).
  ///
  /// If a function, the type option is called as `fn(req)` and the request is
  /// parsed if it return `true`.
  ///
  /// Default = "application/octet-stream"
  pub typ: Option<Either3<String, Vec<String>, Function<'a, Request, bool>>>,

  /// This option, if supplied, is called as `verify(req, res, buf, encoding)`,
  /// where `buf` is a `Buffer` of the raw request body and `encoding` is the
  /// encoding of the request. The parsing can be aborted by throwing an error.
  ///
  /// Default = none
  pub verify: Option<JsVerifyFn<'a>>,
}

struct RawOptions {
  inflate: bool,
  limit: usize,
  typ: Either<Vec<String>, Arc<ThreadsafeParseTypeFn>>,
  verify: Option<Arc<ThreadsafeVerifyFn>>,
}

impl Default for RawOptions {
  fn default() -> Self {
    Self {
      inflate: false,
      limit: 102_400, // 100kb
      typ: Either::A(vec!["application/octet-stream".to_owned()]),
      verify: None,
    }
  }
}

impl<'a> JsRawOptions<'a> {
  fn to_raw_options(&self) -> Result<RawOptions> {
    let mut raw_options = RawOptions::default();

    if let Some(inflate) = self.inflate {
      raw_options.inflate = inflate;
    }

    if let Some(limit) = &self.limit {
      match limit {
        Either::A(limit) => {
          raw_options.limit = *limit as usize;
        }
        Either::B(limit) => {
          let limit = utilities::decimal_to_binary_unit(limit);
          match Byte::from_str(&limit) {
            Ok(limit) => {
              raw_options.limit = limit.as_u64() as usize;
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

    if let Some(media_type) = &self.typ {
      match media_type {
        Either3::A(media_type) => raw_options.typ = Either::A(vec![media_type.to_owned()]),
        Either3::B(media_types) => raw_options.typ = Either::A(media_types.to_owned()),
        Either3::C(media_type_fn) => {
          let tsfn = media_type_fn
            .build_threadsafe_function()
            .build_callback(|ctx: ThreadsafeCallContext<FnArgs<(Request,)>>| Ok(ctx.value))?;
          raw_options.typ = Either::B(Arc::new(tsfn));
        }
      }
    }

    if let Some(verify_fn) = &self.verify {
      let tsfn = verify_fn.build_threadsafe_function().build_callback(
        |ctx: ThreadsafeCallContext<FnArgs<(Request, Response, Buffer, String)>>| Ok(ctx.value),
      )?;
      raw_options.verify = Some(Arc::new(tsfn));
    }

    Ok(raw_options)
  }
}

impl RawOptions {
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

/// This is a built-in middleware function in Express. It parses incoming
/// requests with JSON payloads.
///
/// Returns middleware that only parses JSON and only looks at requests where
/// the `Content-Type` header matches the `type` option. This parser accepts
/// any Unicode encoding of the body and supports automatic inflation of `gzip`
/// and `deflate` encodings.
///
/// A new `body` object containing the parsed data is populated on the
/// `request` object after the middleware (i.e. `req.body`), or `undefined` if
/// there was no body to parse, the `Content-Type` was not matched, or an error
/// occurred.
///
/// > As `req.body`’s shape is based on user-controlled input, all properties
/// > and values in this object are untrusted and should be validated before
/// > trusting. For example, `req.body.foo.toString()` may fail in multiple
/// > ways, for example `foo` may not be there or may not be a string, and
/// > `toString` may not be a function and instead a string or other
/// > user-input.
#[napi]
pub struct RawMiddleware {
  options: RawOptions,
}

#[napi]
impl RawMiddleware {
  #[napi(constructor)]
  pub fn new(options: Option<JsRawOptions>) -> Result<Self> {
    Ok(RawMiddleware {
      options: match options {
        Some(options) => options.to_raw_options()?,
        None => RawOptions::default(),
      },
    })
  }

  #[napi]
  pub async fn run(&self, request: &Request, _response: &Response) -> Result<bool> {
    println!("Raw Middleware | Called!");

    // determine if request should be parsed
    if !self.options.should_parse(request).await? {
      return Ok(true);
    }

    let hyper_request = request.with_inner_mut(|req| req.take_inner())?;
    let (parts, body) = hyper_request.into_parts();
    let mut body_stream = BodyStream::new(Limited::new(body, self.options.limit));

    let mut body = Vec::new();
    while let Some(data) = body_stream.next().await {
      let data = data
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?
        .into_data()
        .map_err(|_| Error::new(Status::GenericFailure, "Encountered a non-data frame."))?;
      body.extend_from_slice(&data);
    }

    request.with_inner_mut(|w_req| {
      w_req.set_inner(HyperRequest::from_parts(parts, BoxBody::new(body_stream)));
      Ok(())
    })?;

    // skip requests without bodies
    if body.is_empty() {
      return Ok(true);
    }

    request.with_inner_mut(|req| {
      req.set_body(Either3::C(body));
      Ok(())
    })?;

    Ok(true)
  }
}
