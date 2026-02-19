use hyper::header::HeaderValue;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};

/// Characters that need to be percent-encoded in RFC 5987 encoding
/// Based on RFC 5987 attr-char definition
const ATTR_CHAR: &AsciiSet = &CONTROLS
  .add(b' ')
  .add(b'"')
  .add(b'%')
  .add(b'\'')
  .add(b'(')
  .add(b')')
  .add(b'*')
  .add(b',')
  .add(b'/')
  .add(b':')
  .add(b';')
  .add(b'<')
  .add(b'=')
  .add(b'>')
  .add(b'?')
  .add(b'@')
  .add(b'[')
  .add(b'\\')
  .add(b']')
  .add(b'{')
  .add(b'}');

/// Generate a RFC 6266 compliant Content-Disposition header value
///
/// This implements both the simple quoted-string format (for ASCII-safe filenames)
/// and the RFC 5987 extended format (for filenames with special or non-ASCII characters)
pub fn content_disposition(filename: &str) -> Result<HeaderValue, String> {
  if filename.is_empty() {
    return Ok(HeaderValue::from_static("attachment"));
  }

  // Check if the filename is "simple" (ASCII-only and no special characters)
  if is_simple_filename(filename) {
    // Use simple quoted-string format: attachment; filename="name.txt"
    let disposition = format!("attachment; filename=\"{}\"", filename);
    HeaderValue::from_str(&disposition).map_err(|e| format!("Invalid header value: {}", e))
  } else {
    // Use both formats for maximum compatibility:
    // 1. A fallback ASCII-only filename (with substitutions)
    // 2. RFC 5987 encoded filename with UTF-8
    let ascii_fallback = to_ascii_fallback(filename);
    let encoded = utf8_percent_encode(filename, ATTR_CHAR).to_string();

    let disposition = format!(
      "attachment; filename=\"{}\"; filename*=UTF-8''{}",
      ascii_fallback, encoded
    );

    HeaderValue::from_str(&disposition).map_err(|e| format!("Invalid header value: {}", e))
  }
}

/// Check if filename only contains safe ASCII characters that don't need encoding
fn is_simple_filename(filename: &str) -> bool {
  filename
    .chars()
    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' || c == ' ')
    && !filename.contains('"')
    && !filename.contains('\\')
}

/// Create an ASCII fallback filename by replacing non-ASCII characters
fn to_ascii_fallback(filename: &str) -> String {
  filename
    .chars()
    .map(|c| {
      if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
        c
      } else if c.is_whitespace() {
        '_'
      } else if c.is_ascii() {
        c
      } else {
        // Replace non-ASCII characters with underscore
        '_'
      }
    })
    .collect::<String>()
    .trim_matches('_')
    .to_string()
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_simple_ascii_filename() {
    let result = content_disposition("test.txt").unwrap();
    assert_eq!(
      result.to_str().unwrap(),
      "attachment; filename=\"test.txt\""
    );
  }

  #[test]
  fn test_filename_with_spaces() {
    let result = content_disposition("my file.txt").unwrap();
    assert_eq!(
      result.to_str().unwrap(),
      "attachment; filename=\"my file.txt\""
    );
  }

  #[test]
  fn test_unicode_filename() {
    let result = content_disposition("测试.txt").unwrap();
    let header = result.to_str().unwrap();

    // Should contain both fallback and RFC 5987 encoded version
    assert!(header.contains("attachment; filename="));
    assert!(header.contains("filename*=UTF-8''"));
    assert!(header.contains("%E6%B5%8B%E8%AF%95.txt"));
  }

  #[test]
  fn test_filename_with_special_chars() {
    let result = content_disposition("file (1).txt").unwrap();
    let header = result.to_str().unwrap();

    // Parentheses require RFC 5987 encoding
    assert!(header.contains("filename*=UTF-8''"));
  }

  #[test]
  fn test_emoji_filename() {
    let result = content_disposition("😀.txt").unwrap();
    let header = result.to_str().unwrap();

    assert!(header.contains("attachment; filename="));
    assert!(header.contains("filename*=UTF-8''%F0%9F%98%80.txt"));
  }

  #[test]
  fn test_empty_filename() {
    let result = content_disposition("").unwrap();
    assert_eq!(result.to_str().unwrap(), "attachment");
  }

  #[test]
  fn test_complex_unicode() {
    let result = content_disposition("файл-документ.pdf").unwrap();
    let header = result.to_str().unwrap();

    // Should have ASCII fallback and proper UTF-8 encoding
    assert!(header.contains("filename*=UTF-8''"));
  }
}
