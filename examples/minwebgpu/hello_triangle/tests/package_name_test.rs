//! Regression coverage for `Cargo.toml`'s package name.
//!
//! Plain-text `include_str!` check, zero dependency on the crate's own (wasm32-gated) items --
//! compiles and runs on any host target, same rationale as `doc_comment_test.rs`.

// test_kind: bug_reproducer(BUG-355)
/// ## Root Cause
/// `Cargo.toml`'s package `name` was `"minwebgpu__"` -- a malformed double-underscore name, not
/// matching the `minwebgpu_<crate>` convention every sibling crate in this same directory
/// follows (`minwebgpu_hello_triangle_quickstart`, `minwebgpu_deffered_rendering`,
/// `minwebgpu_renderer_pbr_scene`). Likely a truncated rename: the intended suffix
/// (`hello_triangle`) was never typed in after the trailing underscore.
/// ## Why Not Caught
/// The malformed name still compiles and links cleanly -- Cargo accepts any valid package-name
/// string, so a wrong-but-well-formed name has no build-time symptom. Nothing cross-checked the
/// declared name against the directory it lives in or its sibling crates' naming convention.
/// ## Fix Applied
/// Renamed the package to `minwebgpu_hello_triangle`, matching the sibling convention, and
/// regenerated `Cargo.lock`'s corresponding entry.
/// ## Prevention
/// A crate's declared package name is a factual claim about its own identity, exactly like a doc
/// comment -- cross-check it against sibling crates' naming convention within the same parent
/// directory rather than trusting it at face value.
/// ## Pitfall
/// A malformed package name (stray trailing underscores, truncated suffix) is silently valid to
/// Cargo -- it produces no compiler error or warning, so this defect class survives indefinitely
/// unless something explicitly checks the name string's content, not just its validity.
#[ test ]
fn cargo_toml_package_name_matches_sibling_convention()
{
  let cargo_toml = include_str!( "../Cargo.toml" );
  assert!(
    !cargo_toml.contains( "name = \"minwebgpu__\"" ),
    "Cargo.toml's package name must not reintroduce the malformed double-underscore name \
    (BUG-355)"
  );
  assert!(
    cargo_toml.contains( "name = \"minwebgpu_hello_triangle\"" ),
    "Cargo.toml's package name must follow the minwebgpu_<crate> sibling convention (BUG-355)"
  );
}
