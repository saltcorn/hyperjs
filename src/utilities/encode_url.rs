fn is_hex(b: u8) -> bool {
  b.is_ascii_hexdigit()
}

fn is_url_safe(b: u8) -> bool {
  matches!(
      b,
      b'A'..=b'Z'
          | b'a'..=b'z'
          | b'0'..=b'9'
          | b'-'
          | b'_'
          | b'.'
          | b'~'
          | b':'
          | b'/'
          | b'?'
          | b'#'
          | b'['
          | b']'
          | b'@'
          | b'!'
          | b'$'
          | b'&'
          | b'\''
          | b'('
          | b')'
          | b'*'
          | b'+'
          | b','
          | b';'
          | b'='
  )
}

pub fn encode_url(input: &str) -> String {
  let bytes = input.as_bytes();
  let mut out = String::with_capacity(bytes.len());
  let mut i = 0;

  while i < bytes.len() {
    let b = bytes[i];

    if b == b'%' {
      if i + 2 < bytes.len() && is_hex(bytes[i + 1]) && is_hex(bytes[i + 2]) {
        // Valid percent-encoded sequence, preserve
        out.push('%');
        out.push(bytes[i + 1] as char);
        out.push(bytes[i + 2] as char);
        i += 3;
      } else {
        // Invalid %, encode it
        out.push_str("%25");
        i += 1;
      }
    } else if is_url_safe(b) {
      out.push(b as char);
      i += 1;
    } else {
      // Encode UTF-8 byte(s)
      let ch = input[i..].chars().next().unwrap_or('\u{FFFD}');
      let mut buf = [0u8; 4];
      for &byte in ch.encode_utf8(&mut buf).as_bytes() {
        out.push_str(&format!("%{:02X}", byte));
      }
      i += ch.len_utf8();
    }
  }

  out
}

#[cfg(test)]
mod tests {
  use super::encode_url;

  #[test]
  fn test_encode_url() {
    assert_eq!(encode_url("/foo bar"), "/foo%20bar".to_owned());
    assert_eq!(encode_url("%20"), "%20".to_owned());
    assert_eq!(encode_url("%zz"), "%25zz".to_owned());
    assert_eq!(encode_url("✓"), "%E2%9C%93".to_owned());
    assert_eq!(encode_url("a%2Fb"), "a%2Fb".to_owned());
  }
}
