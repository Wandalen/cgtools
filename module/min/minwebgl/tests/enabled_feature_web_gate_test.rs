//! Verifies `minwebgl::exec_loop` and `minwebgl::log` resolve under the crate's baseline
//! `enabled` feature -- BUG-279's fix added `mingl/web` and `mingl/web_log` to `enabled`'s own
//! Cargo.toml dependency list, matching what `src/exec_loop.rs`'s unconditional
//! `reuse ::mingl::web::exec_loop;` and `src/log.rs`'s unconditional `reuse ::mingl::web::log;`
//! have always required to compile (`lib.rs` gates neither `layer exec_loop;` nor `layer log;`
//! behind any `#[cfg(...)]` at all, so both are compiled whenever `enabled` is).
//!
//! This test only proves *compile-time module resolution* -- it never calls
//! `exec_loop::run` (which starts a real `requestAnimationFrame` loop and hangs/panics without
//! a live browser `window`) or anything that reaches for the browser console. BUG-279 was a
//! pure E0433 "cannot find `web`/`log` in `mingl`" resolution failure, not a runtime defect, so
//! resolving these paths is the entire regression surface; see this crate's `Cargo.toml` and
//! this session's bug report (`task/bug/completed/279_*.md`) for the full isolated-feature
//! `cargo check` transcripts (before/after) that this single-feature-configuration test file
//! cannot itself reproduce (a `#[test]` binary compiles under exactly one feature selection,
//! and this crate's `texture`/`drawbuffers` layers carry separate, pre-existing, unrelated
//! defects that also surface under `--no-default-features --features enabled` and prevent that
//! exact invocation from ever fully passing regardless of this fix).

// test_kind: bug_reproducer(BUG-279)
/// ## Root Cause
/// `Cargo.toml`'s `enabled` feature listed `dep:web-sys`, `dep:wasm-bindgen`,
/// `dep:wasm-bindgen-futures`, and `dep:js-sys` directly, but never forwarded `mingl/web` (or
/// `mingl/web_log`). `lib.rs` compiles `layer exec_loop;` and `layer log;` unconditionally --
/// no per-layer `#[cfg(...)]` at all -- and both `reuse` straight from `mingl::web::exec_loop`
/// / `mingl::web::log`. But `mingl::web` itself is gated behind `#[cfg(feature = "web")]`, and
/// `mingl::web::log` additionally behind `#[cfg(feature = "web_log")]`, in `mingl`'s own
/// `src/lib.rs` / `src/web.rs`. Selecting `enabled` alone (or any combination omitting this
/// crate's own `web`/`log` features) failed to compile with
/// `E0433: cannot find 'web' in 'mingl'` (`exec_loop.rs`/`log.rs`), then, once `web` alone was
/// added, `E0433: cannot find 'log' in 'web'` (`log.rs`) -- even though `enabled` already
/// hard-requires the exact same underlying dependencies (`web-sys`, `wasm-bindgen`, etc.) that
/// `mingl/web` itself gates, so this crate was never meaningfully usable without them anyway.
///
/// ## Why Not Caught
/// `web`, `log`, `future`, and `file` are all bundled together in this crate's own `default`
/// feature set, and every pre-existing test invocation ran via `--all-features` or plain
/// `cargo test` (default features) -- both always carry `web`+`log` alongside `enabled`, so
/// nothing had ever selected `enabled` without them until this session's isolated-feature spot
/// check (`cargo check -p minwebgl --no-default-features --features enabled`), which surfaced
/// 34 compile errors, two of whose root causes (`exec_loop.rs`, `log.rs`) are this bug --
/// the same isolated-feature-combination technique that found the sibling BUG-274
/// (`diagnostics` omitting `future`/`file`) in this same `Cargo.toml`.
///
/// ## Fix Applied
/// Changed `enabled`'s Cargo.toml feature list to additionally include `mingl/web` and
/// `mingl/web_log`, in `module/min/minwebgl/Cargo.toml`, making the feature graph match what
/// `src/exec_loop.rs` and `src/log.rs` actually, unconditionally need.
///
/// ## Prevention
/// RED state (empirically confirmed): temporarily removing the `"mingl/web"`/`"mingl/web_log"`
/// entries from `enabled`'s list in `Cargo.toml` (this test file left in place) and running
/// `cargo check -p minwebgl --no-default-features --features enabled` genuinely reproduces
/// `error[E0433]: cannot find 'web' in 'mingl'` at `src/exec_loop.rs:10` and
/// `src/log.rs:10` -- verified before finalizing this fix (see the bug report's Verification
/// section for the exact transcripts and the reason a plain `git stash` of the whole
/// `Cargo.toml` was avoided: the file also carries the unrelated, already-applied BUG-274 fix,
/// and stashing the entire file would have reverted that fix's hunk too).
///
/// ## Pitfall
/// A `layer` with no per-item `#[cfg(...)]` reads as "always available," but if its body
/// `reuse`s from an optionally-gated path in another crate, it silently inherits that crate's
/// gate as a real (but unstated) prerequisite of the *outer* feature that always compiles it
/// (`enabled`, here) -- grep every unconditional `layer` for `reuse` targets that cross an
/// optional-feature boundary in the crate it reuses from, and check for *nested* gates one
/// level deeper (here, `web_log` gating `log` from inside the already-gated `web` layer)
/// rather than stopping at the first resolved layer.
#[ test ]
fn exec_loop_and_log_modules_resolve_under_enabled_feature()
{
  // Function-pointer reference only -- proves `minwebgl::exec_loop::run` resolves and
  // monomorphizes without ever invoking it (it would hang/panic outside a live browser).
  // Anonymous `_` (not a named `_run`) deliberately -- clippy's `no_effect_underscore_binding`
  // flags a *named* underscore-prefixed binding that's never read as misleading; the anonymous
  // form correctly signals "evaluate for the resolution/monomorphization side effect only".
  let _ : fn( fn( f64 ) -> bool ) = minwebgl::exec_loop::run::< fn( f64 ) -> bool >;

  // Pure-data enum reference only -- proves `minwebgl::log` resolves through the full
  // `minwebgl::log` -> `mingl::web::log` -> `browser_log::log` -> `::log::*` reuse chain,
  // without touching the browser console.
  let _ = minwebgl::log::Level::Info;
}
