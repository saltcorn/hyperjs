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
pub struct JsTextOptions<'a> {
  /// Specify the default character set for the text content if the charset is
  /// not specified in the `Content-Type` header of the request.
  ///
  /// Default = "utf-8"
  pub default_charset: Option<String>,

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
  /// this can be an extension name (like `txt`), a mime type (like
  /// `text/plain`), or a mime type with a wildcard (like `*/*` or `text/*`).
  /// If a function, the type option is called as `fn(req)` and the request is
  /// parsed if it returns a truthy value.
  ///
  /// Default = "text/plain"
  pub typ: Option<String>,

  /// This option, if supplied, is called as `verify(req, res, buf, encoding)`,
  /// where `buf` is a `Buffer` of the raw request body and `encoding` is the
  /// encoding of the request. The parsing can be aborted by throwing an error.
  pub verify: Option<JsVerifyFn<'a>>,
}

impl<'a> JsTextOptions<'a> {
  fn to_text_options(&self) -> Result<TextOptions> {
    let mut text_options = TextOptions::default();

    if let Some(default_charset) = &self.default_charset {
      text_options.default_charset = default_charset.to_owned();
    }

    if let Some(inflate) = self.inflate {
      text_options.inflate = inflate;
    }

    if let Some(limit) = &self.limit {
      match limit {
        Either::A(limit) => {
          text_options.limit = *limit as usize;
        }
        Either::B(limit) => {
          let limit = utilities::decimal_to_binary_unit(limit);
          match Byte::from_str(&limit) {
            Ok(limit) => {
              text_options.limit = limit.as_u64() as usize;
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

    if let Some(content_type) = &self.typ {
      text_options.typ = content_type.to_owned();
    }

    if let Some(verify_fn) = &self.verify {
      let tsfn = verify_fn.build_threadsafe_function().build_callback(
        |ctx: ThreadsafeCallContext<FnArgs<(Request, Response, Buffer, String)>>| Ok(ctx.value),
      )?;
      text_options.verify = Some(Arc::new(tsfn));
    }

    Ok(text_options)
  }
}

struct TextOptions {
  default_charset: String,
  inflate: bool,
  limit: usize,
  typ: String,
  verify: Option<Arc<ThreadsafeVerifyFn>>,
}

impl Default for TextOptions {
  fn default() -> Self {
    Self {
      default_charset: "utf-8".to_owned(),
      inflate: true,
      limit: 102_400, // 100kb
      typ: "text/plain".to_owned(),
      verify: None,
    }
  }
}

impl TextOptions {
  fn should_parse(&self, request: &Request) -> Result<bool> {
    let req_content_type = match request.get(hyper::http::header::CONTENT_TYPE.to_string())? {
      Either::A(val) => val,
      Either::B(_) => return Ok(false),
    };
    Ok(utilities::type_is(&req_content_type, &[&self.typ]).is_some())
  }
}

/// This is a built-in middleware function in Express. It parses incoming
/// request payloads into a string.
///
/// Returns middleware that parses all bodies as a string and only looks at
/// requests where the `Content-Type` header matches the `type` option. This
/// parser accepts any Unicode encoding of the body and supports automatic
/// inflation of `gzip` and `deflate` encodings.
///
/// A new `body` string containing the parsed data is populated on the
/// `request` object after the middleware (i.e. `req.body`), or `undefined` if
/// there was no body to parse, the `Content-Type` was not matched, or an error
/// occurred.
///
/// > As `req.body`’s shape is based on user-controlled input, all properties
/// > and values in this object are untrusted and should be validated before
/// > trusting. For example, `req.body.trim()` may fail in multiple ways, for
/// > example stacking multiple parsers `req.body` may be from a different
/// > parser. Testing that `req.body` is a string before calling string methods
/// > is recommended.
#[napi]
pub struct TextMiddleware {
  options: TextOptions,
}

#[napi]
impl TextMiddleware {
  #[napi(constructor)]
  pub fn new(options: Option<JsTextOptions>) -> Result<Self> {
    Ok(TextMiddleware {
      options: match options {
        Some(options) => options.to_text_options()?,
        None => TextOptions::default(),
      },
    })
  }

  #[napi]
  pub async fn run(&self, request: &Request, response: &Response) -> Result<bool> {
    log::debug!("Text Middleware | Called!");

    // determine if request should be parsed
    let should_parse = self.options.should_parse(request)?;

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
            self.options.default_charset.to_owned(), // TODO: Pass in actual encoding value
          )
            .into(),
        )
        .await?;
    }

    // TODO: Support multiple text encodings. See iconv-lite npm package.
    //     : See encoding_rs crate

    let req_inner =
      String::from_utf8(body).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    request.with_inner_mut(|w_req| {
      w_req.set_body(Either3::A(req_inner));
      Ok(())
    })?;

    Ok(true)
  }
}
