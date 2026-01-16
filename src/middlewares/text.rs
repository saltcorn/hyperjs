use std::sync::Arc;

use futures::StreamExt;
use http_body_util::{BodyStream, Limited};
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

#[napi]
pub struct JsTextOptions {
  default_charset: String,
  inflate: bool,
  limit: i64,
  typ: String,
  verify: Option<Arc<ThreadsafeVerifyFn>>,
}

struct TextOptions {
  default_charset: String,
  inflate: bool,
  limit: i64,
  typ: String,
  verify: Option<Arc<ThreadsafeVerifyFn>>,
}

impl Default for TextOptions {
  fn default() -> Self {
    Self {
      default_charset: "utf-8".to_owned(),
      inflate: false,
      limit: 102_400, // 100kb
      typ: "text/plain".to_owned(),
      verify: None,
    }
  }
}

impl From<&JsTextOptions> for TextOptions {
  fn from(value: &JsTextOptions) -> Self {
    Self {
      default_charset: value.default_charset.to_owned(),
      inflate: value.inflate,
      limit: value.limit,
      typ: value.typ.to_owned(),
      verify: value.verify.to_owned(),
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

#[napi(object)]
pub struct TextOptionsNewParams {
  pub default_charset: Option<String>,
  pub inflate: Option<bool>,
  pub limit: Option<i64>,
  #[napi(js_name = "type")]
  pub typ: Option<String>,
}

#[napi]
impl JsTextOptions {
  #[napi(constructor)]
  pub fn new(options: TextOptionsNewParams) -> Result<Self> {
    Ok(Self {
      default_charset: options.default_charset.unwrap_or("utf-8".to_owned()),
      inflate: options.inflate.unwrap_or(false),
      limit: options.limit.unwrap_or(102_400), // 100kb
      typ: options.typ.unwrap_or("text/plain".to_owned()),
      verify: None,
    })
  }

  #[napi]
  pub fn verify(
    &mut self,
    verify_fn: Function<FnArgs<(Request, Response, Buffer, String)>, ()>,
  ) -> Result<()> {
    let tsfn = verify_fn.build_threadsafe_function().build_callback(
      |ctx: ThreadsafeCallContext<FnArgs<(Request, Response, Buffer, String)>>| Ok(ctx.value),
    )?;
    self.verify = Some(Arc::new(tsfn));
    Ok(())
  }
}

#[napi]
pub struct TextMiddleware {
  options: TextOptions,
}

#[napi]
impl TextMiddleware {
  #[napi(constructor)]
  pub fn new(options: Option<&JsTextOptions>) -> Self {
    TextMiddleware {
      options: options.map(|options| options.into()).unwrap_or_default(),
    }
  }

  #[napi]
  pub async fn run(&self, request: &Request, response: &Response) -> Result<bool> {
    println!("Text Middleware | Called!");

    // determine if request should be parsed
    let should_parse = self.options.should_parse(request)?;

    let hyper_request = request.with_inner_mut(|req| req.take_inner())?;
    let mut body_stream = BodyStream::new(Limited::new(hyper_request, self.options.limit as usize));

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

    // determine if request should be parsed
    if !should_parse {
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
            "".to_owned(),
          )
            .into(),
        )
        .await?;
    }

    // TODO: Support multiple text encodings. See iconv-lite npm package.
    //     : See encoding_rs crate

    let req_inner =
      String::from_utf8(body).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

    request.with_inner_mut(|req| {
      req.set_body(Either::A(req_inner));
      Ok(())
    })?;
    Ok(true)
  }
}
