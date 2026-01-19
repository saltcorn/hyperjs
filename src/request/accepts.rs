use std::collections::HashMap;

use headers_accept::Accept;
use hyper::header::ACCEPT;
use mediatype::MediaType;
use napi::bindgen_prelude::*;
use napi_derive::napi;

use crate::utilities;

use super::{Request, WrappedRequest};

#[napi]
impl Request {
  /// Checks if the specified content types are acceptable, based on the
  /// request’s Accept HTTP header field. The method returns the best match, or
  /// if none of the specified content types is acceptable, returns false (in
  /// which case, the application should respond with 406 "Not Acceptable").
  ///
  /// The type value may be a single MIME type string (such as
  /// "application/json"), an extension name such as "json", a comma-delimited
  /// list, or an array. For a list or array, the method returns the best match
  /// (if any).
  ///
  /// ```javascript
  /// // Accept: text/html
  /// req.accepts('html')
  /// // => "html"
  ///
  /// // Accept: text/*, application/json
  /// req.accepts('html')u
  /// // => "html"
  /// req.accepts('text/html')
  /// // => "text/html"
  /// req.accepts(['json', 'text'])
  /// // => "json"
  /// req.accepts('application/json')
  /// // => "application/json"
  ///
  /// // Accept: text/*, application/json
  /// req.accepts('image/png')
  /// req.accepts('png')
  /// // => false
  ///
  /// // Accept: text/*;q=.5, application/json
  /// req.accepts(['html', 'json'])
  /// // => "json"
  /// ```
  ///
  /// For more information, or if you have issues or concerns, see
  /// [accepts](https://github.com/expressjs/accepts).
  #[napi]
  pub fn accepts(
    &self,
    types: Either<String, Vec<String>>,
  ) -> Result<Option<Either<String, Vec<String>>>> {
    let types = match types {
      Either::A(r#type) => vec![r#type],
      Either::B(types) => types,
    };
    self.with_inner(|request| request.accepts(types))
  }
}

impl WrappedRequest {
  pub fn accepts(&self, types: Vec<String>) -> Result<Option<Either<String, Vec<String>>>> {
    let accept_values = self.inner()?.headers().get(ACCEPT);

    // no accept header, return first given type
    let Some(client_accept_types) = accept_values else {
      return Ok(types.first().cloned().map(Either::A));
    };

    let Ok(client_accept_types) = client_accept_types.to_str() else {
      return Ok(None);
    };

    let accept: Accept = client_accept_types
      .parse()
      .map_err(|e: headers_core::Error| Error::new(Status::GenericFailure, e.to_string()))?;

    // no types, return all requested types
    if types.is_empty() {
      return Ok(Some(Either::B(
        accept
          .media_types()
          .map(|mt| mt.as_str().to_owned())
          .collect(),
      )));
    }

    let mut available = Vec::new();

    let mut normalized_types = HashMap::new();

    for media_type in types {
      match utilities::guess_media_type(&media_type) {
        Some(guessed_media_type) => {
          normalized_types.insert(guessed_media_type.to_string(), media_type);
        }
        // Guessing from a media type with a character such as * returns None.
        // This block included the media type as is into normalized_types.
        None => {
          if let Ok(mt) = MediaType::parse(&media_type) {
            normalized_types.insert(mt.to_string(), media_type);
          }
        }
      }
    }

    for media_type in normalized_types.keys() {
      let media_type = MediaType::parse(media_type)
        .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;
      available.push(media_type);
    }

    let accepts = accept
      .negotiate(available.iter().collect::<Vec<_>>())
      .and_then(|mt| normalized_types.get(&mt.to_string()))
      .map(|typ| Either::A(typ.to_string()));

    Ok(accepts)
  }
}

#[cfg(test)]
mod tests {
  use headers_core::HeaderValue;
  use hyper::header::ACCEPT;
  use hyper::http::Request as LibRequest;
  use napi::Either;

  use crate::request::WrappedRequest;
  use crate::utilities::empty;

  fn assert_accept_result(
    req_accept_val: &str,
    accepts_params: Vec<String>,
    expected_accept_result: Option<Either<String, Vec<String>>>,
  ) {
    let accept_header = HeaderValue::from_str(req_accept_val).unwrap();
    let mut req = LibRequest::new(empty());
    req.headers_mut().insert(ACCEPT, accept_header);
    let w_req: WrappedRequest = req.into();

    let accept_result = match w_req.accepts(accepts_params).unwrap() {
      Some(accept_result) => Some(accept_result),
      None => match expected_accept_result.is_none() {
        true => return,
        false => {
          panic!(
            "Mismatch between returned and expected return types. left=None, r={expected_accept_result:?}"
          );
        }
      },
    };

    let (accept_result, expected_accept_result) = match (&accept_result, &expected_accept_result) {
      (Some(v1), Some(v2)) => (v1, v2),
      (None, None) => return,
      _ => panic!(
        "Mismatch between returned and expected return types. left={accept_result:?}, r={expected_accept_result:?}"
      ),
    };

    match (accept_result, expected_accept_result) {
      (Either::A(accept_result), Either::A(expected_accept_result)) => {
        assert_eq!(accept_result, expected_accept_result)
      }
      (Either::B(accept_result), Either::B(expected_accept_result)) => {
        assert_eq!(accept_result, expected_accept_result)
      }
      _ => panic!(
        "Mismatch between returned and expected return types. left={accept_result:?}, r={expected_accept_result:?}"
      ),
    }
  }

  #[test]
  fn accepts() {
    assert_accept_result(
      "text/html",
      vec!["html".to_owned()],
      Some(Either::A("html".to_owned())),
    );

    assert_accept_result(
      "text/*, application/json",
      vec!["html".to_owned()],
      Some(Either::A("html".to_owned())),
    );

    assert_accept_result(
      "text/*, application/json",
      vec!["text/html".to_owned()],
      Some(Either::A("text/html".to_owned())),
    );

    assert_accept_result(
      "text/*, application/json",
      vec!["json".to_owned(), "text".to_owned()],
      Some(Either::A("json".to_owned())),
    );

    assert_accept_result(
      "text/*, application/json",
      vec!["application/json".to_owned()],
      Some(Either::A("application/json".to_owned())),
    );

    assert_accept_result(
      "text/*, application/json",
      vec!["image/png".to_owned()],
      None,
    );

    assert_accept_result("text/*, application/json", vec!["png".to_owned()], None);

    assert_accept_result(
      "text/*;q=.5, application/json",
      vec!["html".to_owned(), "json".to_owned()],
      Some(Either::A("json".to_owned())),
    );
  }
}
