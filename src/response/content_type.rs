use hyper::header::{HeaderValue, CONTENT_TYPE};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::Response;

#[napi]
impl Response {
  /// Sets the `Content-Type` HTTP header to the MIME type as determined by the specified `type`. If `type` contains the "/" character,
  /// then it sets the `Content-Type` to the exact value of `type`, otherwise it is assumed to be a file extension and the MIME type
  /// is looked up using the `from_ext()` method of the mime_guess crate.
  ///
  /// ```javascript
  /// res.type('.html') // => 'text/html'
  /// res.type('html') // => 'text/html'
  /// res.type('json') // => 'application/json'
  /// res.type('application/json') // => 'application/json'
  /// res.type('png') // => image/png:
  /// ```
  ///
  /// Aliased as `res.contentType(type)`.
  #[napi(js_name = "type")]
  pub fn typ(&mut self, typ: String) -> Result<()> {
    let mut inner = self.unwrap_inner_or_default();
    let typ = typ.trim_start_matches('.');
    let header_value = match typ.contains("/") {
      true => {
        HeaderValue::from_str(typ).map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?
      }

      false => {
        let media_type = mime_guess::from_ext(typ).first().ok_or(Error::new(
          Status::InvalidArg,
          format!("Unable to determine the media type from the extension '{typ}'"),
        ))?;
        HeaderValue::from_str(media_type.as_ref())
          .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?
      }
    };
    inner.headers_mut().insert(CONTENT_TYPE, header_value);
    self.set_inner(inner)
  }

  #[napi]
  pub fn content_type(&mut self, typ: String) -> Result<()> {
    self.typ(typ)
  }
}

#[cfg(test)]
mod tests {
  use hyper::header::CONTENT_TYPE;

  use super::Response;

  #[test]
  fn content_type() {
    let mut response = Response::new();

    response.typ(".html".to_owned()).unwrap();
    let inner = response.inner().unwrap();
    let content_type_value = inner
      .headers()
      .get(CONTENT_TYPE.as_str())
      .unwrap()
      .to_str()
      .unwrap();
    assert_eq!(content_type_value, "text/html");

    response.typ("html".to_owned()).unwrap();
    let inner = response.inner().unwrap();
    let content_type_value = inner
      .headers()
      .get(CONTENT_TYPE.as_str())
      .unwrap()
      .to_str()
      .unwrap();
    assert_eq!(content_type_value, "text/html");

    response.typ("json".to_owned()).unwrap();
    let inner = response.inner().unwrap();
    let content_type_value = inner
      .headers()
      .get(CONTENT_TYPE.as_str())
      .unwrap()
      .to_str()
      .unwrap();
    assert_eq!(content_type_value, "application/json");

    response.typ("application/json".to_owned()).unwrap();
    let inner = response.inner().unwrap();
    let content_type_value = inner
      .headers()
      .get(CONTENT_TYPE.as_str())
      .unwrap()
      .to_str()
      .unwrap();
    assert_eq!(content_type_value, "application/json");

    response.typ("png".to_owned()).unwrap();
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
