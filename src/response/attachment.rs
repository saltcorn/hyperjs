use std::path::Path;

use hyper::header::{HeaderValue, CONTENT_DISPOSITION};
use napi::bindgen_prelude::*;
use napi_derive::napi;

use super::Response;

#[napi]
impl Response {
  /// Sets the HTTP response Content-Disposition header field to “attachment”. If a file_path is given, then it sets the
  /// Content-Type based on the extension name via res.type(), and sets the Content-Disposition “filename=” parameter.
  ///
  /// ```javascript
  /// res.attachment()
  /// // Content-Disposition: attachment
  ///
  /// res.attachment('path/to/logo.png')
  /// // Content-Disposition: attachment; filename="logo.png"
  /// // Content-Type: image/png
  /// ```
  #[napi]
  pub fn attachment(&mut self, file_path: Option<String>) -> Result<()> {
    let mut inner = self.unwrap_inner_or_default();
    match file_path {
      None => {
        let header_value = HeaderValue::from_str("attachment")
          .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        inner
          .headers_mut()
          .insert(CONTENT_DISPOSITION, header_value);
        self.set_inner(inner)
      }
      Some(file_path) => {
        let path = Path::new(&file_path);
        let file_name = path
          .file_name()
          .ok_or(Error::new(
            Status::InvalidArg,
            "Missing filename in provide path.",
          ))?
          .display();
        let header_value = HeaderValue::from_str(&format!(r#"attachment; filename="{file_name}""#))
          .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
        inner
          .headers_mut()
          .insert(CONTENT_DISPOSITION, header_value);
        let file_extension = path.extension().ok_or(Error::new(
          Status::InvalidArg,
          "Missing file extension in provided path.",
        ))?;
        self.set_inner(inner)?;
        self.typ(file_extension.display().to_string())
      }
    }
  }
}

#[cfg(test)]
mod tests {
  use hyper::header::{CONTENT_DISPOSITION, CONTENT_TYPE};

  use super::Response;

  #[test]
  fn attachment_no_value() {
    let mut response = Response::new();
    response.attachment(None).unwrap();
    let inner = response.inner().unwrap();
    let content_disposition_value = inner.headers().get(CONTENT_DISPOSITION.as_str()).unwrap();
    assert_eq!(content_disposition_value.to_str().unwrap(), "attachment");
  }

  #[test]
  fn attachment_with_value() {
    let mut response = Response::new();
    response
      .attachment(Some("path/to/logo.png".to_owned()))
      .unwrap();
    let inner = response.inner().unwrap();
    let content_disposition_value = inner.headers().get(CONTENT_DISPOSITION.as_str()).unwrap();
    assert_eq!(
      content_disposition_value.to_str().unwrap(),
      r#"attachment; filename="logo.png""#
    );
    let content_type_value = inner.headers().get(CONTENT_TYPE.as_str()).unwrap();
    assert_eq!(content_type_value.to_str().unwrap(), "image/png");
  }
}
