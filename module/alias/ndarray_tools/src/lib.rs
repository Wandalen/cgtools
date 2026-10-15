//! Ndarray-based tools and utilities for numerical computing.
#![ doc( html_root_url = "https://docs.rs/ndarray_tools/latest/ndarray_tools/" ) ]
#![ cfg_attr( doc, doc = include_str!( concat!( env!( "CARGO_MANIFEST_DIR" ), "/", "readme.md" ) ) ) ]
#![ cfg_attr( not( doc ), doc = "Ndarray-based tools and utilities for numerical computing" ) ]

// Fix(BUG-170): `use ::mod_interface::mod_interface;` and the `mod_interface!` invocation below
// were both unconditional, even though `mod_interface` and `ndarray_cg` -- the only crate this
// alias reuses -- are optional dependencies gated behind the `enabled` feature. Building with
// `--no-default-features` failed immediately with `E0432: unresolved import 'mod_interface'`.
// Root cause: `enabled` gates every dependency this crate has (`dep:mod_interface`,
// `dep:ndarray_cg`), but nothing in `lib.rs` was gated to match -- unlike `browser_log`
// (BUG-169), where the macro invocation itself was correctly gated and only a sibling item was
// missed, here neither the import nor the invocation had any gate at all.
// Pitfall: a crate whose Cargo.toml gates 100% of its dependencies behind one feature needs that
// same feature on every unconditional item in `lib.rs` that references those dependencies --
// checked individually, since Cargo.toml's own gating gives no compile-time guarantee lib.rs
// matches it.
#[ cfg( feature = "enabled" ) ]
use ::mod_interface::mod_interface;

mod private
{
  // use super::*;
}

#[ cfg( feature = "enabled" ) ]
crate::mod_interface!
{
  /// Reusing main crate.
  reuse ::ndarray_cg;
}
