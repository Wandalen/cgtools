# Fix animation crate's Sequencer/Tween bugs, wrong API doc table, and macro-export lint

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** 📝 (Draft)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/animation
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

Fix 3 logic bugs identified in `animation`'s `Sequencer`/`Tween` code during the workspace audit (P2 —
remaining logic bugs, Fix-in-place), separately correct the crate's readme/doc API table, which was
found to describe an API shape that doesn't match the real one, and separately fix a compiler
future-incompatibility warning on `impl_easing_function`. **Carried forward from the audit triage plan —
exact file/line citations for the 3 bugs and the specific wrong table entries are not re-verified in this
filing pass; re-confirm against current `module/helper/animation/src/` and its readme before touching
anything.** The future-incompatibility item, by contrast, was directly confirmed this session
(2026-08-09) via `cargo check -p scene_script --target wasm32-unknown-unknown --lib`, which pulls in
`animation` as a transitive dependency:

- **Where:** `#[ macro_export ] macro_rules! impl_easing_function { ... }` is defined in
  `module/helper/animation/src/easing/base.rs:45-67`. Its only call site is
  `module/helper/animation/src/easing/cubic/bezier.rs`, which imports it via
  `use crate::{ impl_easing_function, Animatable };` (line 5) and invokes it 24 times (lines 114-144) to
  generate one `EasingBuilder` struct per named easing curve (`EaseInSine`, `EaseOutQuad`, etc.).
  `impl_easing_function` is never re-exported through `mod_interface!`'s `orphan use { ... }` block in
  either file — it is purely an internal code-generation macro, never part of the crate's public API.
- **Why it fires:** `macro_expanded_macro_exports_accessed_by_absolute_paths` (rust-lang/rust#52234).
  `#[macro_export]` binds a macro at the crate root via a legacy mechanism that predates Rust's
  path-based (2018+) macro resolution. Referencing that crate-root binding through an explicit absolute
  path (`use crate::impl_easing_function;`) trips a known compatibility gap that is slated to become a
  hard error. Since the macro is 100% crate-internal (no downstream crate ever needs `#[macro_export]`'s
  cross-crate reach), the fix is to stop relying on that mechanism entirely rather than work around it.
- **Fix (verified working this session, then reverted — not left applied; see History):** two `use`
  statements are needed in `base.rs`, not one — `macro_rules!` items are textually scoped, so a single
  outer re-export isn't enough on its own.
  1. Remove `#[ macro_export ]` from the `macro_rules! impl_easing_function` definition.
  2. Immediately after the macro body, still **inside** `mod private`, add `pub( crate ) use
     impl_easing_function;` — this is required first: textual macro scope ends at the `mod private`
     boundary, so without it the macro has no path-nameable identity for the next step to find (confirmed
     by testing step 3 alone first: `error[E0432]: unresolved import` — "no `impl_easing_function` in
     `easing::base::private`").
  3. **Outside** `mod private`, next to the `crate::mod_interface! { ... }` block at the bottom of the
     file, add `pub( crate ) use private::impl_easing_function;` — this is the piece that actually makes
     `crate::easing::base::impl_easing_function` resolvable from other files, mirroring how
     `mod_interface!`'s own `orphan use { EasingBuilder, EasingFunction, Linear, Step };` re-exports
     everything else in this file.
  4. In `bezier.rs`, change line 5's `use crate::{ impl_easing_function, Animatable };` to
     `use crate::Animatable;` plus folding `impl_easing_function` into the existing
     `use crate::easing::{ base::{ EasingFunction, EasingBuilder } };` block (lines 6-13) as
     `base::{ EasingFunction, EasingBuilder, impl_easing_function }`.

  Verified clean with all four edits together: `cargo build -p animation -vv` shows zero warnings (was
  previously flagged), `cargo test -p animation` passes 24/24, `cargo check -p animation --target
  wasm32-unknown-unknown --lib` passes with exit 0. `git diff --stat module/helper/animation/` confirms
  no trace was left behind after reverting for this filing.

Bundled as one task since all three concerns are small and confined to the same crate; split into
separate tasks at pickup if any turns out to be larger than expected.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P2 (remaining logic
  bugs) tier, merged with a P5 (doc drift) item for the same crate, Fix-in-place bucket.
- **[2026-08-09]** `UPDATED` — Added a third bundled concern: a
  `macro_expanded_macro_exports_accessed_by_absolute_paths` future-incompatibility warning on
  `impl_easing_function` (rust-lang/rust#52234), discovered while verifying `scene_script`'s wasm32
  build (which depends on `animation` transitively). Root-caused, and the fix was actually applied and
  verified in-session (clean build, 24/24 tests, wasm32 check all passed) to confirm the recipe is
  correct — the first draft of the fix (a single `use` statement) was wrong; testing caught that it
  needs two. Then reverted (`git diff --stat` confirmed byte-identical to HEAD) since the user asked
  only to file this for later pickup, not to apply it now. Fix recipe in Goal is empirically verified,
  not speculative.
