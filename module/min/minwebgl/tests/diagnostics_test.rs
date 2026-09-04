//! Verifies `diagnostics`'s `obj` re-export resolves when the `diagnostics` feature is
//! selected without directly, separately selecting `future`/`file` -- the BUG-274 fix added
//! `future`/`file` to `diagnostics`'s own Cargo.toml dependency list so the feature is
//! self-sufficient, matching what `src/diagnostics.rs`'s unconditional
//! `own use crate::model::obj;` has always required to compile.

#[ cfg( feature = "diagnostics" ) ]
use minwebgl::diagnostics::obj::reports_make;

// test_kind: bug_reproducer(BUG-274)
/// ## Root Cause
/// `Cargo.toml`'s `diagnostics` feature was declared as
/// `diagnostics = [ "mingl/diagnostics", "model_obj" ]`. `src/diagnostics.rs` (the file this
/// feature gates in) unconditionally does `own use crate::model::obj;`, but `lib.rs` gates
/// `layer model;` -- the module `obj` lives under -- behind
/// `all( feature = "future", feature = "file" )`, a predicate `diagnostics` never forwarded.
/// Selecting `diagnostics` without also separately selecting both `future` and `file` left
/// `crate::model` configured out, failing with `E0432: unresolved import 'crate::model'`.
///
/// ## Why Not Caught
/// `diagnostics`, `future`, and `file` are all bundled together in this crate's own `default`
/// feature set, and every existing test invocation runs via `--all-features` or plain
/// `cargo test` (default features) -- both always carry `future`/`file` alongside `diagnostics`,
/// so nothing had ever selected `diagnostics` without them until this session's isolated-feature
/// spot check (`cargo check -p minwebgl --no-default-features --features enabled,web,log,constants,diagnostics`).
///
/// ## Fix Applied
/// Changed `diagnostics = [ "mingl/diagnostics", "model_obj" ]` to
/// `diagnostics = [ "mingl/diagnostics", "model_obj", "future", "file" ]` in
/// `module/min/minwebgl/Cargo.toml`, making the feature graph match what
/// `src/diagnostics.rs` actually, unconditionally needs.
///
/// ## Prevention
/// RED state (empirically confirmed): reverting only the `Cargo.toml` half of this fix
/// (`git stash push -- module/min/minwebgl/Cargo.toml`) with this test still in place, then
/// running
/// `cargo test -p minwebgl --no-default-features --features enabled,web,log,constants,diagnostics`
/// genuinely fails to compile with `E0432: unresolved import 'crate::model'` -- verified before
/// finalizing this fix.
///
/// ## Pitfall
/// A feature that forwards one prerequisite (`model_obj`, which gates the *content* inside
/// `model/obj.rs`) can still be incomplete if the module *containing* that content
/// (`layer model;`) is gated behind an entirely different predicate (`future`+`file`) that
/// nothing forces the first feature to also select -- `--all-features` and this crate's own
/// `default` bundle never distinguish the two because they always enable every predicate at once.
#[ cfg( feature = "diagnostics" ) ]
#[ test ]
fn diagnostics_obj_reexport_resolves_under_diagnostics_feature_alone()
{
  let reports = reports_make( &[], &[] );
  assert!( reports.is_empty(), "reports_make with empty input must return an empty Vec" );
}
