//! ## Root Cause
//! `image_load` in `src/main.rs` accepted a `path : &str` parameter but ignored it for two of
//! its three uses: it called `image_element_create( "tileset.png" )` with a hardcoded literal
//! instead of `path`, and `image.set_id( "tileset.png" )` likewise. Since `image_element_create`
//! resolves its argument against the app root (no `static/` prefix), the element's initial `src`
//! pointed at a URL that 404s on every page load -- a real, wasted network request, immediately
//! overwritten a few lines later by the correctly-computed `url` built from `path`.
//!
//! ## Why Not Caught
//! No test file existed for this crate -- it is a `fn main()`-only WebGL demo binary with no lib
//! target, and `image_load` touches `web_sys`/DOM APIs that only run inside an actual browser, so
//! it cannot be exercised by a plain `cargo test`. The single current call site happens to pass
//! `"static/tileset.png"`, whose filename component coincidentally matches the hardcoded
//! `"tileset.png"` literal used for the DOM id, masking the bug for that one purpose; the wrong
//! initial `src` request has no visible symptom besides an extra failed network request, easy to
//! miss without inspecting the browser's network panel.
//!
//! ## Fix Applied
//! `image_element_create` is now called with `path` (the real, resolvable path) instead of the
//! hardcoded literal. `image.set_id` now derives its id from `path`'s filename component
//! (`path.rsplit('/').next()`), matching the bare-filename ids `texture_array_prepare`'s
//! `get_element_by_id` lookups already expect elsewhere in this file.
//!
//! ## Prevention
//! This test parses `src/main.rs`'s own text via `include_str!` and asserts `image_element_create`
//! is called with the `path` parameter (not a hardcoded string literal).
//!
//! ## Pitfall
//! A parameter that is genuinely used for one of several related purposes (here, `set_src`) reads
//! as "used" at a glance -- silently ignoring it for the other purposes (element creation, id)
//! hides easily behind a single call site whose literal happens to coincide with the real value.

// BUG-338 task/bug/XXX_wfc_image_load_ignores_path_parameter.md -- reproducer for image_load()
// calling image_element_create() with a hardcoded "tileset.png" literal instead of its own
// `path` parameter, causing a wasted/failed network request on every page load.
// test_kind: bug_reproducer(BUG-338)
#[ test ]
fn image_load_passes_path_parameter_to_image_element_create()
{
  let source = include_str!( "../src/main.rs" );

  assert!(
    source.contains( "image_element_create( path )" ),
    "image_load must call image_element_create with its own `path` parameter, not a hardcoded literal"
  );
  assert!(
    !source.contains( "image_element_create( \"tileset.png\" )" ),
    "image_load must not hardcode \"tileset.png\" when creating the image element -- \
     use the `path` parameter so the resolved URL actually matches what the caller requested"
  );
}
