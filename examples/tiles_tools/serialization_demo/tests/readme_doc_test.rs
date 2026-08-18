//! Doc-text regression tests for `serialization_demo/readme.md`.
//!
//! Pure `include_str!` + substring assertions -- no library target needed
//! (this crate is binary-only), matching this session's established
//! `include_str!` precedent for doc-only defects that resist black-box
//! runtime testing.

// test_kind: bug_reproducer(BUG-304)
/// ## Root Cause
/// `readme.md` enumerated 5 features (3-format serialization, config
/// management, save-file management, version-compatibility checking) but
/// omitted "Compression support" -- the 6th item in `src/main.rs`'s own
/// module doc comment list. `compression_demonstrate()` exists in
/// `src/main.rs` and is actively called from `main()`, so the omission
/// undercounted a real, exercised feature.
/// ## Why Not Caught
/// This crate is binary-only (`src/main.rs`, no `src/lib.rs`) and had zero
/// pre-existing test coverage, so nothing tied the readme's enumerated
/// feature list to `src/main.rs`'s own module doc comment or to which
/// `*_demonstrate` functions `main()` actually calls.
/// ## Fix Applied
/// Added "compression support" to the readme's feature list so it matches
/// all 6 items in `src/main.rs`'s own module doc comment.
/// ## Prevention
/// An enumerated feature-list claim needs its own doc-text regression test
/// reading the file's actual prose -- a claim like this can silently
/// undercount again if the readme and the module doc comment are edited
/// independently.
/// ## Pitfall
/// A prose list introduced as "exercises: A, B, C, and D" is a falsifiable
/// completeness claim, unlike a loose descriptive summary -- treat it as
/// testable doc text, not as informal color commentary exempt from
/// verification.
#[ test ]
fn readme_lists_compression_support_alongside_other_five_features()
{
  let readme = include_str!( "../readme.md" );
  assert!
  (
    readme.contains( "compression support" ),
    "serialization_demo/readme.md's feature list must include compression support -- \
    src/main.rs's own module doc comment lists it as feature 6 of 6, and \
    compression_demonstrate() is actually called from main() (BUG-304)"
  );
  assert!
  (
    readme.contains( "JSON, Binary, RON" )
    && readme.contains( "configuration management" )
    && readme.contains( "save-file management" )
    && readme.contains( "version-compatibility checking" ),
    "serialization_demo/readme.md must still list the other 5 features alongside compression \
    support (BUG-304)"
  );
}
