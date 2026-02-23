use mime_guess::Mime;
use std::str::FromStr;

/// Checks if a given content type matches any of the provided type patterns.
///
/// This function provides similar functionality to the npm `type-is` package.
/// It checks if the content_type matches any of the patterns in `types`.
///
/// Patterns can be:
/// - Exact MIME types: "application/json"
/// - Wildcard types: "application/*", "*/json", "*/*"
/// - Extension shortcuts: "json", "html", "png"
/// - Extension with dot: ".json", ".html"
///
/// # Arguments
/// * `content_type` - The content type to check (e.g., "application/json; charset=utf-8")
/// * `types` - A slice of type patterns to match against
///
/// # Returns
/// * `Some(String)` - The matching pattern if found
/// * `None` - If no pattern matches
///
/// # Examples
/// ```
/// let result = type_is("application/json", &["json"]);
/// assert_eq!(result, Some("json".to_string()));
///
/// let result = type_is("text/html; charset=utf-8", &["html", "json"]);
/// assert_eq!(result, Some("html".to_string()));
///
/// let result = type_is("image/png", &["image/*"]);
/// assert_eq!(result, Some("image/*".to_string()));
/// ```
pub fn type_is(content_type: &str, types: &[&str]) -> Option<String> {
  // Parse the content type, ignoring parameters like charset
  let parsed_mime = parse_content_type(content_type)?;

  for type_pattern in types {
    if matches_pattern(&parsed_mime, type_pattern) {
      return Some(type_pattern.to_string());
    }
  }

  None
}

/// Parse a content type string into a Mime type, handling parameters
fn parse_content_type(content_type: &str) -> Option<Mime> {
  // Remove leading/trailing whitespace
  let content_type = content_type.trim();

  if content_type.is_empty() {
    return None;
  }

  // Split on semicolon to remove parameters like charset
  let mime_part = content_type.split(';').next()?.trim();

  // Try to parse as MIME type
  Mime::from_str(mime_part).ok()
}

/// Check if a MIME type matches a given pattern
fn matches_pattern(mime: &Mime, pattern: &str) -> bool {
  let pattern = pattern.trim();

  // Handle extension shortcuts (with or without dot)
  if !pattern.contains('/') {
    let ext = pattern.trim_start_matches('.');
    return matches_extension(mime, ext);
  }

  // Handle wildcard patterns
  if pattern.contains('*') {
    return matches_wildcard(mime, pattern);
  }

  // Handle exact MIME type match
  if let Ok(pattern_mime) = Mime::from_str(pattern) {
    return mime.type_() == pattern_mime.type_() && mime.subtype() == pattern_mime.subtype();
  }

  false
}

/// Check if a MIME type matches a file extension
fn matches_extension(mime: &Mime, ext: &str) -> bool {
  // Get all MIME types for this extension
  let guesses = mime_guess::from_ext(ext);

  // Check if any of the guessed types match our MIME type
  for guess in guesses.iter() {
    if mime.type_() == guess.type_() && mime.subtype() == guess.subtype() {
      return true;
    }
  }

  false
}

/// Check if a MIME type matches a wildcard pattern
fn matches_wildcard(mime: &Mime, pattern: &str) -> bool {
  let parts: Vec<&str> = pattern.split('/').collect();

  if parts.len() != 2 {
    return false;
  }

  let (pattern_type, pattern_subtype) = (parts[0], parts[1]);

  // Match type
  let type_matches = pattern_type == "*" || pattern_type == mime.type_().as_str();

  // Match subtype
  let subtype_matches = pattern_subtype == "*" || pattern_subtype == mime.subtype().as_str();

  type_matches && subtype_matches
}

/// Convenient wrapper that accepts multiple content types
/// Returns the first matching type pattern or None
#[allow(unused)]
pub fn type_is_multi(content_types: &[&str], types: &[&str]) -> Option<String> {
  for content_type in content_types {
    if let Some(matched) = type_is(content_type, types) {
      return Some(matched);
    }
  }
  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_exact_mime_type() {
    assert_eq!(
      type_is("application/json", &["application/json"]),
      Some("application/json".to_string())
    );
    assert_eq!(
      type_is("text/html", &["text/html"]),
      Some("text/html".to_string())
    );
  }

  #[test]
  fn test_extension_shortcuts() {
    assert_eq!(
      type_is("application/json", &["json"]),
      Some("json".to_string())
    );
    assert_eq!(type_is("text/html", &["html"]), Some("html".to_string()));
    assert_eq!(type_is("image/png", &["png"]), Some("png".to_string()));
  }

  #[test]
  fn test_extension_with_dot() {
    assert_eq!(
      type_is("application/json", &[".json"]),
      Some(".json".to_string())
    );
    assert_eq!(type_is("text/html", &[".html"]), Some(".html".to_string()));
  }

  #[test]
  fn test_wildcard_type() {
    assert_eq!(
      type_is("image/png", &["image/*"]),
      Some("image/*".to_string())
    );
    assert_eq!(
      type_is("image/jpeg", &["image/*"]),
      Some("image/*".to_string())
    );
    assert_eq!(type_is("text/plain", &["image/*"]), None);
  }

  #[test]
  fn test_wildcard_subtype() {
    assert_eq!(
      type_is("application/json", &["*/json"]),
      Some("*/json".to_string())
    );
    assert_eq!(
      type_is("text/json", &["*/json"]),
      Some("*/json".to_string())
    );
  }

  #[test]
  fn test_wildcard_all() {
    assert_eq!(
      type_is("application/json", &["*/*"]),
      Some("*/*".to_string())
    );
    assert_eq!(type_is("image/png", &["*/*"]), Some("*/*".to_string()));
  }

  #[test]
  fn test_multiple_types() {
    assert_eq!(
      type_is("application/json", &["html", "json", "xml"]),
      Some("json".to_string())
    );
    assert_eq!(
      type_is("text/html", &["json", "html", "xml"]),
      Some("html".to_string())
    );
  }

  #[test]
  fn test_content_type_with_parameters() {
    assert_eq!(
      type_is("application/json; charset=utf-8", &["json"]),
      Some("json".to_string())
    );
    assert_eq!(
      type_is("text/html; charset=utf-8", &["text/html"]),
      Some("text/html".to_string())
    );
  }

  #[test]
  fn test_no_match() {
    assert_eq!(type_is("application/json", &["html", "xml"]), None);
    assert_eq!(type_is("image/png", &["video/*"]), None);
  }

  #[test]
  fn test_invalid_content_type() {
    assert_eq!(type_is("", &["json"]), None);
    assert_eq!(type_is("invalid", &["json"]), None);
  }

  #[test]
  fn test_multi_content_types() {
    assert_eq!(
      type_is_multi(&["text/plain", "application/json"], &["json"]),
      Some("json".to_string())
    );
    assert_eq!(
      type_is_multi(&["image/png", "text/html"], &["html", "json"]),
      Some("html".to_string())
    );
  }

  #[test]
  fn test_first_match_wins() {
    // Should return the first matching pattern in the types array
    assert_eq!(
      type_is("application/json", &["json", "application/json", "*/*"]),
      Some("json".to_string())
    );
  }
}
