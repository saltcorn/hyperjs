use std::str::FromStr;

use mime_guess::Mime;

use super::guess_media_type;

/// Compare a `value` content-type with `types`.
/// Each `type` can be an extension like `html`,
/// a special shortcut like `multipart` or `urlencoded`,
/// or a mime type.
///
/// If no types match, `None` is returned.
/// Otherwise, the first `type` that matches is returned.
pub fn type_is<'a>(value: &'a str, types: &'a [&'a str]) -> Option<String> {
  // return None if value is invalid
  let value_guess = Mime::from_str(value).ok()?;

  // no types, return the content type
  if types.is_empty() {
    return Some(value_guess.to_string());
  }

  for typ in types {
    // Guess the typ's media type from the provided string
    let Some(type_media_type) = guess_media_type(typ)
      // Parse type into a media type as is
      .or_else(|| Mime::from_str(typ).ok())
    else {
      continue;
    };

    // return typ if the value and typ media types are equal
    if type_media_type == value_guess {
      return Some((*typ).to_owned());
    }

    // return value if value and typ media type suffixes are equal and typ
    // starts with *
    if typ.starts_with("*") && type_media_type.suffix() == value_guess.suffix() {
      return Some(value.to_owned());
    }

    // return value if value and typ media type types are equal and typ ends
    // with *
    if typ.ends_with("*") && type_media_type.type_() == value_guess.type_() {
      return Some(value.to_owned());
    }
  }

  None
}

#[cfg(test)]
mod tests {
  use super::type_is;

  #[test]
  fn test_type_is() {
    let result = type_is("application/json", &["json"]);
    assert_eq!(result, Some("json".to_owned()));

    let result = type_is("application/json", &["html", "json"]);
    assert_eq!(result, Some("json".to_owned()));

    let result = type_is("application/json", &["application/*"]);
    assert_eq!(result, Some("application/json".to_owned()));

    let result = type_is("application/json", &["application/json"]);
    assert_eq!(result, Some("application/json".to_owned()));

    let result = type_is("application/json", &["html"]);
    assert_eq!(result, None);
  }
}
