use std::path::Path;

pub fn contains_dot_file(path: &Path) -> bool {
  path.components().any(|component| {
    component
      .as_os_str()
      .to_str()
      .map(|s| s.len() > 1 && s.starts_with('.'))
      .unwrap_or(false)
  })
}
