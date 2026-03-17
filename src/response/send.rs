use bytes::Bytes;
use hyper::{
  StatusCode,
  header::{CONTENT_LENGTH, CONTENT_TYPE, HeaderValue, TRANSFER_ENCODING},
};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Response, WrappedResponse};

#[napi]
impl Response {
  /// Sends the HTTP response.
  ///
  /// The body parameter can be a Buffer object, a String, an object, Boolean, or
  /// an Array. For example:
  ///
  /// ```javascript
  /// res.send(Buffer.from('whoop'))
  /// res.send({ some: 'json' })
  /// res.send('<p>some html</p>')
  /// res.status(404).send('Sorry, we cannot find that!')
  /// res.status(500).send({ error: 'something blew up' })
  /// ```
  ///
  /// This method performs many useful tasks for simple non-streaming responses:
  /// For example, it automatically assigns the Content-Length HTTP response
  /// header field and provides automatic HEAD and HTTP cache freshness support.
  ///
  /// When the parameter is a Buffer object, the method sets the Content-Type
  /// response header field to "application/octet-stream", unless previously
  /// defined as shown below:
  ///
  /// ```javascript
  /// res.set('Content-Type', 'text/html')
  /// res.send(Buffer.from('<p>some html</p>'))
  /// ```
  ///
  /// When the parameter is a String, the method sets the Content-Type to
  /// "text/html":
  ///
  /// ```javascript
  /// res.send('<p>some html</p>')
  /// ```
  ///
  /// When the parameter is an Array or Object, Express responds with the JSON
  /// representation:
  ///
  /// ```javascript
  /// res.send({ user: 'tobi' })
  /// res.send([1, 2, 3])
  /// ```
  #[napi]
  pub fn send(
    &mut self,
    body: Either6<String, i64, bool, Null, Buffer, Object>,
    env: Env,
  ) -> Result<()> {
    self.with_inner(|response| response.send(body, env))
  }
}

impl WrappedResponse {
  pub fn send(
    &mut self,
    body: Either6<String, i64, bool, Null, Buffer, Object>,
    env: Env,
  ) -> Result<()> {
    let mut chunk = match body {
      // set `Content-Type` to text/html if the provided body is a string
      Either6::A(value) => {
        if self.inner()?.headers().get(CONTENT_TYPE).is_none() {
          self.content_type("html".to_owned())?
        }
        Either::A(value)
      }
      // set `Content-Type` to application/json if the provided body is an
      // object, number or boolean.
      Either6::B(value) => return self.json(Either5::B(value), env),
      Either6::C(value) => return self.json(Either5::C(value), env),
      Either6::D(_) => return self.json(Either5::A("".to_owned()), env),
      // set the `Content-Type` to application/octet-stream if the provided
      // body is a bytes array
      Either6::E(value) => {
        log::debug!("RS: Received buffer. Data: {:?}", value.iter().as_slice());
        if self.inner()?.headers().get(CONTENT_TYPE).is_none() {
          self.content_type("bin".to_owned())?
        }
        Either::B(value)
      }
      Either6::F(value) => return self.json(Either5::D(value), env),
    };

    // write strings in utf-8
    if let Either::A(_) = &chunk
      && let Some(content_type) = self.inner()?.headers().get(CONTENT_TYPE)
    {
      let content_type_str = content_type
        .to_str()
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
      if !content_type_str.contains("charset") {
        let content_type = HeaderValue::from_str(&format!("{content_type_str}; charset=utf-8"))
          .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        self
          .inner()?
          .headers_mut()
          .insert(CONTENT_TYPE, content_type);
      }
    }

    // TODO: Support configuring if ETag should be generated

    // TODO: Support freshness detection and header

    // strip irrelevant headers
    let status_code = self.inner()?.status();
    if status_code == StatusCode::NOT_MODIFIED || status_code == StatusCode::NO_CONTENT {
      let headers = self.inner()?.headers_mut();
      headers.remove(CONTENT_TYPE);
      headers.remove(CONTENT_LENGTH);
      headers.remove(TRANSFER_ENCODING);
      chunk = Either::A("".to_owned());
    }

    let chunk = match chunk {
      Either::A(v) => Bytes::copy_from_slice(v.as_bytes()),
      Either::B(v) => Bytes::copy_from_slice(v.as_ref()),
    };

    // TODO: Seal response from further modification
    self.end(Some(chunk))
  }
}
