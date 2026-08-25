//!
//! # Readers and writers for embroidery formats
//!

mod private
{
  /// Truncates `s` to at most `max_bytes` bytes, backing off to the nearest
  /// preceding UTF-8 character boundary so the result is always valid UTF-8
  /// (never splits a multi-byte character).
  // Fix(BUG-498)
  // Root cause: `pec::writer::pec_header_write`'s name field and
  // `pes::writer::{pes_string16_write,pes_string8_write}` all truncated a
  // `&str` by raw byte length (`&s.as_bytes()[ ..len ]`) with no UTF-8
  // character-boundary check, silently splitting a multi-byte character in
  // half and embedding invalid UTF-8 into the written file whenever a name
  // string's truncation point happened to fall mid-character.
  // Pitfall: `&str`'s own byte length (`.len()`) and its *character* count are
  // different units -- slicing by a raw byte count is only safe when the
  // slice point is independently known to fall on a character boundary
  // (`str::is_char_boundary`), never merely because the count itself looks
  // small enough.
  #[ must_use ]
  pub fn str_truncate_char_boundary( s : &str, max_bytes : usize ) -> &str
  {
    if s.len() <= max_bytes
    {
      return s;
    }

    let mut end = max_bytes;
    while end > 0 && !s.is_char_boundary( end )
    {
      end -= 1;
    }

    &s[ ..end ]
  }
}

crate::mod_interface!
{
  layer pec;
  layer pes;

  own use str_truncate_char_boundary;
}
