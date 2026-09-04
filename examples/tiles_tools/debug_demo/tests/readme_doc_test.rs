//! Doc-text regression tests for `debug_demo`'s module doc comment
//! (`src/main.rs`) and `readme.md`.
//!
//! Pure `include_str!` + substring assertions -- no library target needed
//! (this crate is binary-only), matching this session's established
//! `include_str!` precedent for doc-only defects that resist black-box
//! runtime testing.

// test_kind: bug_reproducer(BUG-303)
/// ## Root Cause
/// Both `src/main.rs`'s module doc comment ("ASCII art rendering and SVG
/// export capabilities") and `readme.md` ("both ASCII-art and SVG
/// rendering of the same grid state") claimed the demo actively exercises
/// SVG export. Every `svg_export`/`csv_export` call site in `main.rs` is
/// commented out (e.g. `// square_grid.svg_export(...)`); only
/// `ascii_render()` is actually called at runtime.
/// ## Why Not Caught
/// This crate is binary-only (`src/main.rs`, no `src/lib.rs`) and had zero
/// pre-existing test coverage, so nothing tied either doc claim to which
/// export call sites were actually live vs. commented out.
/// ## Fix Applied
/// Corrected both `src/main.rs`'s module doc comment and `readme.md` to
/// describe only the ASCII-art rendering the demo actually performs,
/// dropping the false SVG-export claim from both.
/// ## Prevention
/// A doc claim naming a specific capability ("SVG export") needs its own
/// doc-text regression test reading the file's actual prose -- re-enabling
/// the commented-out `svg_export`/`csv_export` calls in a future change
/// should be paired with restoring this claim, not the other way around
/// (claiming a capability that was quietly disabled).
/// ## Pitfall
/// Commented-out code that still contains a real, once-working call site
/// is easy to miss when auditing doc claims -- `grep` for the claimed
/// capability's call site itself (not just its mention in doc text) to
/// confirm it is actually live, not dead.
#[ test ]
fn main_rs_module_doc_comment_does_not_claim_svg_export()
{
  let main_rs = include_str!( "../src/main.rs" );
  assert!
  (
    !main_rs.contains( "//! - ASCII art rendering and SVG export capabilities" ),
    "debug_demo/src/main.rs's module doc comment must not claim SVG export -- every \
    svg_export/csv_export call site in this file is commented out, only ascii_render() actually \
    runs (BUG-303)"
  );
  assert!
  (
    main_rs.contains( "ASCII art rendering" ),
    "debug_demo/src/main.rs's module doc comment must still describe the ASCII art rendering the \
    demo actually performs (BUG-303)"
  );
}

// test_kind: bug_reproducer(BUG-303)
/// ## Root Cause
/// See `main_rs_module_doc_comment_does_not_claim_svg_export` above -- same
/// defect, duplicated into `readme.md`'s own prose ("both ASCII-art and
/// SVG rendering of the same grid state").
/// ## Why Not Caught
/// See above -- no test tied readme.md's claim to `main.rs`'s actual
/// (commented-out) SVG export call sites either.
/// ## Fix Applied
/// Corrected `readme.md` to describe only ASCII-art rendering, matching
/// `main.rs`'s actual runtime behavior.
/// ## Prevention
/// See above.
/// ## Pitfall
/// See above -- the same false claim was copy-pasted into 2 files (module
/// doc comment and readme.md), so both needed their own direct doc-text
/// assertion; fixing one without the other would leave a real,
/// independently-checkable contradiction behind.
#[ test ]
fn readme_does_not_claim_svg_rendering()
{
  let readme = include_str!( "../readme.md" );
  assert!
  (
    !readme.contains( "SVG" ),
    "debug_demo/readme.md must not claim SVG rendering -- the demo only performs ASCII-art \
    rendering at runtime, every SVG export call site in src/main.rs is commented out (BUG-303)"
  );
  assert!
  (
    readme.contains( "ASCII-art rendering" ),
    "debug_demo/readme.md must still describe the ASCII-art rendering the demo actually performs \
    (BUG-303)"
  );
}
