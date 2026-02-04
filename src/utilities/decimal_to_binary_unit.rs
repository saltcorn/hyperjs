pub fn decimal_to_binary_unit(input: &str) -> String {
  let input_lower = input.to_lowercase();

  // Map of decimal units to binary units
  let unit_map = [
    ("kb", "kib"),
    ("mb", "mib"),
    ("gb", "gib"),
    ("tb", "tib"),
    ("pb", "pib"),
    ("eb", "eib"),
  ];

  // Try to find and replace the unit
  for (decimal, binary) in unit_map {
    if input_lower.ends_with(decimal) {
      // Preserve the original case pattern
      let replacement = if input.chars().any(|c| c.is_uppercase()) {
        // If original had uppercase, use uppercase for binary unit
        binary.to_uppercase()
      } else {
        binary.to_string()
      };

      let prefix = &input[..input.len() - decimal.len()];
      return format!("{}{}", prefix, replacement);
    }
  }

  // If no decimal unit found, return original string
  input.to_string()
}
