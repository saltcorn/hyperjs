use cookie::{Cookie, CookieJar, Key};
use headers_core::HeaderValue;
use napi::{Error, Result, Status};
use serde_json::{Map as JsonMap, Value as JsonValue};

#[derive(Debug)]
pub struct RequestCookies {
  pub encrypted: JsonValue,
  pub unencrypted: JsonValue,
}

pub fn extract_cookies<T: AsRef<[u8]>>(
  secrets: &[T],
  cookie_header_value: &HeaderValue,
) -> Result<RequestCookies> {
  let cookie_header_value_str = get_cookie_header_value_as_str(cookie_header_value)?;
  let cookie_jar = parse_cookies(cookie_header_value_str)?;
  let private_cookie_jars = secrets
    .iter()
    .map(|secret| Key::from(secret.as_ref()))
    .map(|key| cookie_jar.private(&key))
    .collect::<Vec<_>>();
  let mut private_cookies = JsonMap::new();
  let mut public_cookies = JsonMap::new();
  for cookie in cookie_jar.iter() {
    let mut private_cookie = None;
    for private_cookie_jar in &private_cookie_jars {
      if let Some(decrypted_private_cookie) = private_cookie_jar.get(cookie.name()) {
        private_cookie = Some(decrypted_private_cookie);
        break;
      }
    }
    match private_cookie {
      Some(private_cookie) => {
        private_cookies.insert(
          private_cookie.name().to_owned(),
          private_cookie.value().to_owned().into(),
        );
      }
      None => {
        public_cookies.insert(cookie.name().to_owned(), cookie.value().to_owned().into());
      }
    }
  }
  Ok(RequestCookies {
    encrypted: JsonValue::Object(private_cookies),
    unencrypted: JsonValue::Object(public_cookies),
  })
}

fn get_cookie_header_value_as_str(cookie_header_value: &HeaderValue) -> Result<&str> {
  cookie_header_value.to_str().map_err(|_| {
    Error::new(
      Status::GenericFailure,
      "Expected Cookie header value to be a string. Found a byte array.",
    )
  })
}

pub fn parse_cookies(cookie_header_value: &str) -> Result<CookieJar> {
  let mut cookies = CookieJar::new();

  for cookie in Cookie::split_parse_encoded(cookie_header_value) {
    let cookie = cookie.map_err(|e| {
      Error::new(
        Status::GenericFailure,
        format!("Error parsing Cookie header: {e}"),
      )
    })?;
    cookies.add(cookie.into_owned());
  }

  Ok(cookies)
}

#[cfg(test)]
mod tests {
  use super::*;
  use cookie::{Cookie, CookieJar, Key};
  use headers_core::HeaderValue;

  // ============================================================================
  // Helper Functions for Tests
  // ============================================================================

  /// Create a HeaderValue from a string
  fn header_value_from_str(s: &str) -> HeaderValue {
    HeaderValue::from_str(s).unwrap()
  }

  /// Create an encrypted cookie using the cookie crate
  fn create_encrypted_cookie(name: &str, value: &str, secret: &str) -> String {
    let mut jar = CookieJar::new();
    let key = Key::from(secret.as_bytes());
    let mut private_jar = jar.private_mut(&key);
    private_jar.add(Cookie::new(name, value).into_owned());

    // Extract the encrypted cookie value
    jar
      .iter()
      .map(|c| format!("{}={}", c.name(), c.value()))
      .collect::<Vec<_>>()
      .join("; ")
  }

  fn elongate_secret(secret: &str) -> String {
    secret.repeat(20)
  }

  // ============================================================================
  // Tests for get_cookie_header_value_as_str
  // ============================================================================

  #[test]
  fn test_get_cookie_header_value_as_str_valid_string() {
    let header = header_value_from_str("session=abc123");
    let result = get_cookie_header_value_as_str(&header);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "session=abc123");
  }

  #[test]
  fn test_get_cookie_header_value_as_str_empty_string() {
    let header = header_value_from_str("");
    let result = get_cookie_header_value_as_str(&header);

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "");
  }

  #[test]
  fn test_get_cookie_header_value_as_str_invalid_utf8() {
    // Create a HeaderValue with invalid UTF-8 bytes
    let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
    let header = HeaderValue::from_bytes(&invalid_bytes).unwrap();
    let result = get_cookie_header_value_as_str(&header);

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.status, Status::GenericFailure);
    assert!(
      err
        .reason
        .contains("Expected Cookie header value to be a string")
    );
  }

  // ============================================================================
  // Tests for parse_cookies
  // ============================================================================

  #[test]
  fn test_parse_cookies_single_cookie() {
    let cookie_str = "session=abc123";
    let result = parse_cookies(cookie_str);

    assert!(result.is_ok());
    let jar = result.unwrap();
    assert_eq!(jar.iter().count(), 1);

    let cookie = jar.get("session").unwrap();
    assert_eq!(cookie.name(), "session");
    assert_eq!(cookie.value(), "abc123");
  }

  #[test]
  fn test_parse_cookies_multiple_cookies() {
    let cookie_str = "session=abc123; user=john; theme=dark";
    let result = parse_cookies(cookie_str);

    assert!(result.is_ok());
    let jar = result.unwrap();
    assert_eq!(jar.iter().count(), 3);

    assert_eq!(jar.get("session").unwrap().value(), "abc123");
    assert_eq!(jar.get("user").unwrap().value(), "john");
    assert_eq!(jar.get("theme").unwrap().value(), "dark");
  }

  #[test]
  fn test_parse_cookies_empty_string() {
    let cookie_str = "";
    let result = parse_cookies(cookie_str);

    assert!(result.is_ok());
    let jar = result.unwrap();
    assert_eq!(jar.iter().count(), 0);
  }

  #[test]
  fn test_parse_cookies_encoded_values() {
    let cookie_str = "message=hello%20world; special=%3D%3D%3D";
    let result = parse_cookies(cookie_str);

    assert!(result.is_ok());
    let jar = result.unwrap();

    // The cookie crate should decode these automatically
    let message = jar.get("message").unwrap();
    assert_eq!(message.value(), "hello world");

    let special = jar.get("special").unwrap();
    assert_eq!(special.value(), "===");
  }

  #[test]
  fn test_parse_cookies_with_whitespace() {
    let cookie_str = "  session=abc123  ;  user=john  ";
    let result = parse_cookies(cookie_str);

    assert!(result.is_ok());
    let jar = result.unwrap();
    assert_eq!(jar.iter().count(), 2);
  }

  #[test]
  fn test_parse_cookies_duplicate_names() {
    // Multiple cookies with the same name - last one should win
    let cookie_str = "session=first; session=second";
    let result = parse_cookies(cookie_str);

    assert!(result.is_ok());
    let jar = result.unwrap();

    // CookieJar may contain both, but get() returns the last added
    let session = jar.get("session").unwrap();
    assert_eq!(session.value(), "second");
  }

  #[test]
  fn test_parse_cookies_invalid_format() {
    let cookie_str = "this-is-not-a-valid-cookie";
    let result = parse_cookies(cookie_str);

    // Depending on cookie crate behavior, this may error or be skipped
    // The current implementation should catch parse errors
    assert!(result.is_err());
  }

  #[test]
  fn test_parse_cookies_empty_value() {
    let cookie_str = "empty=; nonempty=value";
    let result = parse_cookies(cookie_str);

    assert!(result.is_ok());
    let jar = result.unwrap();

    let empty = jar.get("empty").unwrap();
    assert_eq!(empty.value(), "");

    let nonempty = jar.get("nonempty").unwrap();
    assert_eq!(nonempty.value(), "value");
  }

  #[test]
  fn test_parse_cookies_special_characters_in_name() {
    let cookie_str = "my-cookie=value; my_cookie=value2";
    let result = parse_cookies(cookie_str);

    assert!(result.is_ok());
    let jar = result.unwrap();
    assert!(jar.get("my-cookie").is_some());
    assert!(jar.get("my_cookie").is_some());
  }

  // ============================================================================
  // Tests for extract_cookies
  // ============================================================================

  #[test]
  fn test_extract_cookies_no_cookies() {
    let header = header_value_from_str("");
    let secret = elongate_secret("secret123");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();
    let encrypted_cookies = cookies.encrypted.as_object().unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();
    assert_eq!(encrypted_cookies.len(), 0);
    assert_eq!(unencrypted_cookies.len(), 0);
  }

  #[test]
  fn test_extract_cookies_only_unsigned() {
    let header = header_value_from_str("session=abc123; user=john");
    let secret = elongate_secret("secret123");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();

    let encrypted_cookies = cookies.encrypted.as_object().unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();
    assert_eq!(encrypted_cookies.len(), 0);
    assert_eq!(unencrypted_cookies.len(), 2);
    assert_eq!(
      unencrypted_cookies
        .get("session")
        .unwrap()
        .as_str()
        .unwrap(),
      "abc123"
    );
    assert_eq!(
      unencrypted_cookies.get("user").unwrap().as_str().unwrap(),
      "john"
    );
  }

  #[test]
  fn test_extract_cookies_with_signed_cookie() {
    let secret = elongate_secret("my_secret_key_32_bytes_long!!");

    // Create an encrypted cookie
    let encrypted = create_encrypted_cookie("secure_session", "secret_value", &secret);
    let header = header_value_from_str(&encrypted);

    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();

    let encrypted_cookies = cookies.encrypted.as_object().unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();
    // The encrypted cookie should be decrypted and placed in signed
    assert_eq!(encrypted_cookies.len(), 1);
    assert_eq!(
      cookies
        .encrypted
        .get("secure_session")
        .unwrap()
        .as_str()
        .unwrap(),
      "secret_value"
    );
    assert_eq!(unencrypted_cookies.len(), 0);
  }

  #[test]
  fn test_extract_cookies_mixed_signed_and_unsigned() {
    let secret = elongate_secret("my_secret_key_32_bytes_long!!");

    // Create an encrypted cookie
    let encrypted = create_encrypted_cookie("secure_session", "secret_value", &secret);

    // Combine with unsigned cookies
    let combined = format!("{}; public=value; another=test", encrypted);
    let header = header_value_from_str(&combined);

    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();

    let encrypted_cookies = cookies.encrypted.as_object().unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();
    assert_eq!(encrypted_cookies.len(), 1);
    assert_eq!(
      cookies
        .encrypted
        .get("secure_session")
        .unwrap()
        .as_str()
        .unwrap(),
      "secret_value"
    );

    assert_eq!(unencrypted_cookies.len(), 2);
    assert_eq!(
      cookies.unencrypted.get("public").unwrap().as_str().unwrap(),
      "value"
    );
    assert_eq!(
      cookies
        .unencrypted
        .get("another")
        .unwrap()
        .as_str()
        .unwrap(),
      "test"
    );
  }

  #[test]
  fn test_extract_cookies_multiple_secrets() {
    let secret1 = elongate_secret("secret_key_number_one_32bytes");
    let secret2 = elongate_secret("secret_key_number_two_32bytes");

    // Create cookies encrypted with different secrets
    let encrypted1 = create_encrypted_cookie("cookie1", "value1", &secret1);
    let encrypted2 = create_encrypted_cookie("cookie2", "value2", &secret2);

    // Extract just the cookie values (remove the cookie1= and cookie2= parts)
    // let cookie1_value = encrypted1.split('=').nth(1).unwrap();
    // let cookie2_value = encrypted2.split('=').nth(1).unwrap();

    let combined = format!("{}; {}", encrypted1, encrypted2);
    let header = header_value_from_str(&combined);

    // Both secrets should be able to decrypt their respective cookies
    let secrets = vec![secret1.as_bytes(), secret2.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();

    let encrypted_cookies = cookies.encrypted.as_object().unwrap();
    // Both should be decrypted
    assert_eq!(encrypted_cookies.len(), 2);
    assert_eq!(
      cookies.encrypted.get("cookie1").unwrap().as_str().unwrap(),
      "value1"
    );
    assert_eq!(
      cookies.encrypted.get("cookie2").unwrap().as_str().unwrap(),
      "value2"
    );
  }

  #[test]
  fn test_extract_cookies_wrong_secret() {
    let secret = elongate_secret("my_secret_key_32_bytes_long!!");
    let wrong_secret = elongate_secret("wrong_secret_key_32_bytes_long");

    // Create an encrypted cookie with one secret
    let encrypted = create_encrypted_cookie("secure_session", "secret_value", &secret);
    let header = header_value_from_str(&encrypted);

    // Try to decrypt with wrong secret
    let secrets = vec![wrong_secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();

    let encrypted_cookies = cookies.encrypted.as_object().unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();
    // Should not be decrypted, stays in unsigned
    assert_eq!(encrypted_cookies.len(), 0);
    assert_eq!(unencrypted_cookies.len(), 1);
  }

  #[test]
  fn test_extract_cookies_empty_secrets() {
    let header = header_value_from_str("session=abc123");
    let secrets: Vec<&[u8]> = vec![];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();

    let encrypted_cookies = cookies.encrypted.as_object().unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();
    // No secrets means everything is unsigned
    assert_eq!(encrypted_cookies.len(), 0);
    assert_eq!(unencrypted_cookies.len(), 1);
  }

  #[test]
  fn test_extract_cookies_invalid_header() {
    // Create invalid UTF-8 header
    let invalid_bytes = vec![0xFF, 0xFE, 0xFD];
    let header = HeaderValue::from_bytes(&invalid_bytes).unwrap();

    let secret = elongate_secret("secret");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_err());
  }

  #[test]
  fn test_extract_cookies_malformed_cookie_string() {
    let header = header_value_from_str("not-a-valid-cookie-format");
    let secret = elongate_secret("secret");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    // Should propagate the parsing error
    assert!(result.is_err());
  }

  #[test]
  fn test_extract_cookies_special_characters_in_values() {
    let header = header_value_from_str("special=%3D%3D%3D; emoji=%F0%9F%8D%AA");
    let secret = elongate_secret("secret");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();

    assert_eq!(
      unencrypted_cookies
        .get("special")
        .unwrap()
        .as_str()
        .unwrap(),
      "==="
    );
    assert_eq!(
      unencrypted_cookies.get("emoji").unwrap().as_str().unwrap(),
      "🍪"
    );
  }

  #[test]
  fn test_extract_cookies_preserves_cookie_names() {
    let header = header_value_from_str("Session=value1; SESSION=value2; session=value3");
    let secret = elongate_secret("secret");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();

    // Cookie names are case-sensitive
    // Depending on CookieJar behavior, may have one or multiple
    assert!(!unencrypted_cookies.is_empty());
  }

  #[test]
  fn test_extract_cookies_large_number_of_cookies() {
    // Test with many cookies
    let mut cookie_parts = Vec::new();
    for i in 0..100 {
      cookie_parts.push(format!("cookie{}=value{}", i, i));
    }
    let cookie_str = cookie_parts.join("; ");
    let header = header_value_from_str(&cookie_str);

    let secret = elongate_secret("secret");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();
    assert_eq!(unencrypted_cookies.len(), 100);
  }

  #[test]
  fn test_extract_cookies_json_value_types() {
    let header = header_value_from_str("str=hello; num=123; bool=true");
    let secret = elongate_secret("secret");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();

    // All values should be stored as JSON strings
    assert!(unencrypted_cookies.get("str").unwrap().is_string());
    assert!(unencrypted_cookies.get("num").unwrap().is_string());
    assert!(unencrypted_cookies.get("bool").unwrap().is_string());

    assert_eq!(
      unencrypted_cookies.get("num").unwrap().as_str().unwrap(),
      "123"
    );
  }

  // ============================================================================
  // Integration Tests
  // ============================================================================

  #[test]
  fn test_integration_full_workflow() {
    let secret = elongate_secret("integration_test_secret_key!");

    // Simulate a real request with mixed cookies
    let encrypted = create_encrypted_cookie("user_id", "12345", &secret);
    let combined = format!("{}; session=abc; csrf=xyz", encrypted);
    let header = header_value_from_str(&combined);

    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();
    let encrypted_cookies = cookies.encrypted.as_object().unwrap();
    let unencrypted_cookies = cookies.unencrypted.as_object().unwrap();

    // Verify signed cookie was decrypted
    assert_eq!(encrypted_cookies.len(), 1);
    assert_eq!(
      encrypted_cookies.get("user_id").unwrap().as_str().unwrap(),
      "12345"
    );

    // Verify unsigned cookies remain
    assert_eq!(unencrypted_cookies.len(), 2);
    assert!(unencrypted_cookies.contains_key("session"));
    assert!(unencrypted_cookies.contains_key("csrf"));
  }

  #[test]
  fn test_integration_secret_rotation() {
    let old_secret = elongate_secret("old_secret_key_32_bytes_long!");
    let new_secret = elongate_secret("new_secret_key_32_bytes_long!");

    // Cookie encrypted with old secret
    let encrypted = create_encrypted_cookie("session", "old_session", &old_secret);
    let header = header_value_from_str(&encrypted);

    // Should work with both secrets in rotation
    let secrets = vec![new_secret.as_bytes(), old_secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();
    let encrypted_cookies = cookies.encrypted.as_object().unwrap();

    // Should still decrypt with old secret
    assert_eq!(encrypted_cookies.len(), 1);
    assert_eq!(
      encrypted_cookies.get("session").unwrap().as_str().unwrap(),
      "old_session"
    );
  }

  // ============================================================================
  // Edge Cases and Boundary Tests
  // ============================================================================

  #[test]
  fn test_edge_case_very_long_cookie_value() {
    let long_value = "a".repeat(4000);
    let cookie_str = format!("long={}", long_value);
    let header = header_value_from_str(&cookie_str);

    let secret = elongate_secret("secret");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();
    assert_eq!(
      cookies
        .unencrypted
        .get("long")
        .unwrap()
        .as_str()
        .unwrap()
        .len(),
      4000
    );
  }

  #[test]
  fn test_edge_case_cookie_with_equals_in_value() {
    let cookie_str = "data=key=value";
    let header = header_value_from_str(cookie_str);

    let secret = elongate_secret("secret");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();
    assert_eq!(
      cookies.unencrypted.get("data").unwrap().as_str().unwrap(),
      "key=value"
    );
  }

  #[test]
  fn test_edge_case_cookie_with_semicolon_encoded() {
    let cookie_str = "data=before%3Bafter";
    let header = header_value_from_str(cookie_str);

    let secret = elongate_secret("secret");
    let secrets = vec![secret.as_bytes()];
    let result = extract_cookies(&secrets, &header);

    assert!(result.is_ok());
    let cookies = result.unwrap();
    assert_eq!(
      cookies.unencrypted.get("data").unwrap().as_str().unwrap(),
      "before;after"
    );
  }
}
