use hyper::header::{CONTENT_TYPE, HeaderValue};
use mime_guess::Mime;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::{Response, WrappedResponse};

#[napi]
impl Response {
  /// Sets the `Content-Type` HTTP header to the MIME type as determined by the specified
  /// `type`. If `type` contains the "/" character, then it sets the `Content-Type` to
  /// the exact value of `type`, otherwise it is assumed to be a file extension and the
  /// MIME type is looked up using the `from_ext()` method of the mime_guess crate. When
  /// no mapping is found though `mime_guess::from_ext()`, the type is set to
  /// "application/octet-stream".
  ///
  ///  Examples:
  ///
  ///  ```javascript
  ///      res.type('.html'); // => 'text/html'
  ///      res.type('html'); // => 'text/html'
  ///      res.type('json'); // => 'application/json'
  ///      res.type('application/json'); // => 'application/json'
  ///      res.type('png'); // => image/png
  /// ```
  ///  
  /// Aliased as `res.contentType(type)`.
  #[napi(js_name = "type")]
  pub fn typ(&mut self, typ: String) -> Result<()> {
    self.with_inner(|response| response.content_type(typ))
  }

  #[napi]
  pub fn content_type(&mut self, typ: String) -> Result<()> {
    self.typ(typ)
  }
}

impl WrappedResponse {
  pub fn content_type(&mut self, typ: String) -> Result<()> {
    let typ = typ.trim_start_matches('.');
    let header_value = match typ.contains("/") {
      true => {
        HeaderValue::from_str(typ).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?
      }

      false => {
        let media_type = match mime_guess::from_ext(typ).first() {
          Some(media_type) => media_type,
          None => "application/octet-stream"
            .parse::<Mime>()
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?,
        };
        HeaderValue::from_str(media_type.as_ref())
          .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?
      }
    };
    self
      .inner()?
      .headers_mut()
      .insert(CONTENT_TYPE, header_value);
    Ok(())
  }
}

#[cfg(test)]
mod tests {
  use hyper::header::CONTENT_TYPE;

  use super::WrappedResponse;

  #[test]
  fn content_type() {
    let mut response = WrappedResponse::default();

    response.content_type(".html".to_owned()).unwrap();
    let inner = response.inner().unwrap();
    let content_type_value = inner
      .headers()
      .get(CONTENT_TYPE.as_str())
      .unwrap()
      .to_str()
      .unwrap();
    assert_eq!(content_type_value, "text/html");

    response.content_type("html".to_owned()).unwrap();
    let inner = response.inner().unwrap();
    let content_type_value = inner
      .headers()
      .get(CONTENT_TYPE.as_str())
      .unwrap()
      .to_str()
      .unwrap();
    assert_eq!(content_type_value, "text/html");

    response.content_type("json".to_owned()).unwrap();
    let inner = response.inner().unwrap();
    let content_type_value = inner
      .headers()
      .get(CONTENT_TYPE.as_str())
      .unwrap()
      .to_str()
      .unwrap();
    assert_eq!(content_type_value, "application/json");

    response
      .content_type("application/json".to_owned())
      .unwrap();
    let inner = response.inner().unwrap();
    let content_type_value = inner
      .headers()
      .get(CONTENT_TYPE.as_str())
      .unwrap()
      .to_str()
      .unwrap();
    assert_eq!(content_type_value, "application/json");

    response.content_type("png".to_owned()).unwrap();
    let inner = response.inner().unwrap();
    let content_type_value = inner
      .headers()
      .get(CONTENT_TYPE.as_str())
      .unwrap()
      .to_str()
      .unwrap();
    assert_eq!(content_type_value, "image/png");
  }
}
