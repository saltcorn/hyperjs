use std::{str::FromStr, sync::Arc};

use byte_unit::Byte;
use futures::StreamExt;
use http_body_util::{BodyStream, Limited, combinators::BoxBody};
use hyper::Request as HyperRequest;
use napi::bindgen_prelude::*;
use napi::threadsafe_function::{ThreadsafeCallContext, ThreadsafeFunction};
use napi_derive::napi;

use crate::utilities::{UrlencodedOptions as UrlencodedParseOptions, parse_urlencoded};
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
pub struct JsUrlencodedOptions<'a> {
  /// This option allows to choose between parsing the URL-encoded data with
  /// the `serde_urlencoded` library (when `false`) or the `serde_qs` library
  /// (when `true`). The "extended" syntax allows for rich objects and arrays
  /// to be encoded into the URL-encoded format, allowing for a JSON-like
  /// experience with URL-encoded. For more information, please
  /// [see the serde_qs library](https://docs.rs/serde_qs/1.0.0/serde_qs/index.html).
  ///
  /// Default = false
  pub extended: Option<bool>,

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

  /// This option controls the maximum number of parameters that are allowed in
  /// the URL-encoded data. If a request contains more parameters than this
  /// value, an error will be raised.
  ///
  /// Default = 1000
  pub parameter_limit: Option<u32>,

  /// This is used to determine what media type the middleware will parse. This
  /// option can be a string, array of strings, or a function. If not a
  /// function, `type` option is passed directly to the
  /// [mime_guess](https://docs.rs/mime_guess/latest/mime_guess/) library and
  /// this can be an extension name (like `urlencoded`), a mime type (like
  /// `application/x-www-form-urlencoded`), or a mime type with a wildcard
  /// (like `*/x-www-form-urlencoded`). If a function, the type option is
  /// called as `fn(req)` and the request is parsed if it returns a truthy
  /// value.
  ///
  /// Default = "application/x-www-form-urlencoded"
  pub typ: Option<Either3<String, Vec<String>, Function<'a, Request, bool>>>,

  /// This option, if supplied, is called as `verify(req, res, buf, encoding)`,
  /// where `buf` is a `Buffer` of the raw request body and `encoding` is the
  /// encoding of the request. The parsing can be aborted by throwing an error.
  pub verify: Option<JsVerifyFn<'a>>,

  /// Configure the maximum depth of the `serde_qs` library when extended is
  /// `true`. This allows you to limit the amount of keys that are parsed and
  /// can be useful to prevent certain types of abuse. It is recommended to
  /// keep this value as low as possible.
  ///
  /// Default = 32
  pub depth: Option<u8>,
}

impl<'a> TryFrom<JsUrlencodedOptions<'a>> for UrlencodedOptions {
  type Error = Error;

  fn try_from(value: JsUrlencodedOptions<'a>) -> std::result::Result<Self, Self::Error> {
    let mut urlencoded_options = UrlencodedOptions::default();

    if let Some(extended) = value.extended {
      urlencoded_options.extended = extended;
    }

    if let Some(inflate) = value.inflate {
      urlencoded_options.inflate = inflate;
    }

    if let Some(limit) = &value.limit {
      match limit {
        Either::A(limit) => {
          urlencoded_options.limit = *limit as usize;
        }
        Either::B(limit) => {
          let limit = utilities::decimal_to_binary_unit(limit);
          match Byte::from_str(&limit) {
            Ok(limit) => {
              urlencoded_options.limit = limit.as_u64() as usize;
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

    if let Some(parameter_limit) = value.parameter_limit {
      urlencoded_options.parameter_limit = parameter_limit as usize;
    }

    if let Some(media_type) = &value.typ {
      match media_type {
        Either3::A(media_type) => urlencoded_options.typ = Either::A(vec![media_type.to_owned()]),
        Either3::B(media_types) => urlencoded_options.typ = Either::A(media_types.to_owned()),
        Either3::C(media_type_fn) => {
          let tsfn = media_type_fn
            .build_threadsafe_function()
            .build_callback(|ctx: ThreadsafeCallContext<FnArgs<(Request,)>>| Ok(ctx.value))?;
          urlencoded_options.typ = Either::B(Arc::new(tsfn));
        }
      }
    }

    if let Some(verify_fn) = &value.verify {
      let tsfn = verify_fn.build_threadsafe_function().build_callback(
        |ctx: ThreadsafeCallContext<FnArgs<(Request, Response, Buffer, String)>>| Ok(ctx.value),
      )?;
      urlencoded_options.verify = Some(Arc::new(tsfn));
    }

    if let Some(depth) = value.depth {
      urlencoded_options.depth = depth as usize;
    }

    Ok(urlencoded_options)
  }
}

struct UrlencodedOptions {
  extended: bool,
  inflate: bool,
  limit: usize,
  parameter_limit: usize,
  typ: Either<Vec<String>, Arc<ThreadsafeParseTypeFn>>,
  verify: Option<Arc<ThreadsafeVerifyFn>>,
  depth: usize,
}

impl Default for UrlencodedOptions {
  fn default() -> Self {
    Self {
      extended: false,
      inflate: true,
      limit: 102_400, // 100kb
      parameter_limit: 1000,
      typ: Either::A(vec!["application/x-www-form-urlencoded".to_owned()]),
      verify: None,
      depth: 32,
    }
  }
}

impl UrlencodedOptions {
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

impl From<&UrlencodedOptions> for UrlencodedParseOptions {
  fn from(value: &UrlencodedOptions) -> Self {
    Self {
      extended: value.extended,
      parameter_limit: value.parameter_limit,
      depth: value.depth,
    }
  }
}

/// This is a built-in middleware function in Express. It parses incoming
/// requests with urlencoded payloads.
///
/// Returns middleware that only parses urlencoded bodies and only looks at
/// requests where the `Content-Type` header matches the `type` option. This
/// parser accepts only UTF-8 encoding of the body and supports automatic
/// inflation of `gzip` and `deflate` encodings.
///
/// A new `body` object containing the parsed data is populated on the
/// `request` object after the middleware (i.e. `req.body`), or `undefined` if
/// there was no body to parse, the `Content-Type` was not matched, or an error
/// occurred. This object will contain key-value pairs, where the value can be
/// a string or array (when `extended` is `false`), or any type (when
/// `extended` is `true`).
///
/// > As `req.body`’s shape is based on user-controlled input, all properties
/// > and values in this object are untrusted and should be validated before
/// > trusting. For example, `req.body.foo.toString()` may fail in multiple
/// > ways, for example `foo` may not be there or may not be a string, and
/// > `toString` may not be a function and instead a string or other
/// > user-input.
#[napi]
pub struct UrlencodedMiddleware {
  options: UrlencodedOptions,
}

#[napi]
impl UrlencodedMiddleware {
  #[napi(constructor)]
  pub fn new(options: Option<JsUrlencodedOptions>) -> Result<Self> {
    Ok(UrlencodedMiddleware {
      options: match options {
        Some(options) => UrlencodedOptions::try_from(options)?,
        None => UrlencodedOptions::default(),
      },
    })
  }

  #[napi]
  pub async fn run(&self, request: &Request, response: &Response) -> Result<bool> {
    println!("Urlencoded Middleware | Called!");

    // determine if request should be parsed
    let should_parse = self.options.should_parse(request).await?;

    // determine if request should be parsed
    if !should_parse {
      return Ok(true);
    }

    let hyper_request = request.with_inner_mut(|w_req| w_req.take_inner())?;
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

    if let Some(verify) = self.options.verify.clone() {
      let body_buf = Buffer::from(body.as_slice());
      verify
        .call_async(
          (
            request.to_owned(),
            response.to_owned(),
            body_buf,
            "utf-8".to_owned(),
          )
            .into(),
        )
        .await?;
    }

    let req_inner =
      String::from_utf8(body).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    let parsed_body = parse_urlencoded(req_inner.as_str(), &(&self.options).into())
      .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    request.with_inner_mut(|w_req| {
      w_req.set_body(Either3::B(parsed_body));
      Ok(())
    })?;

    Ok(true)
  }
}
