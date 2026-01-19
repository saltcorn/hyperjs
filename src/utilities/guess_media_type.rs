use mime_guess::{Mime, MimeGuess};

/// Try to guess the media type of a provided content type string
pub fn guess_media_type(content_type: &str) -> Option<Mime> {
  MimeGuess::from_ext(content_type)
    .first()
    .or_else(|| MimeGuess::from_path(content_type).first())
}
