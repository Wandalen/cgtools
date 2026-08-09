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

Fix logic bugs identified in `animation`'s `Sequencer`/`Tween` code during the workspace audit (P2 —
remaining logic bugs, Fix-in-place), correct the crate's readme API-reference table (which described an
API shape that didn't match the real one), and separately fix a compiler future-incompatibility warning
on `impl_easing_function`. Originally filed without re-verified citations; re-audited with exact
file:line citations and fixed in full on 2026-08-09 for the bugs and doc concerns — the macro-export
lint concern remains deliberately deferred, unchanged from the 2026-08-08 decision recorded below.

### Bugs (re-audited and fixed, 2026-08-09)

Each fixed in-place with a `Fix(TASK-015)` / `Root cause` / `Pitfall` source comment, plus a dedicated
`bug_reproducer(TASK-015)` test:

1. **`[Tween<T>; N]::duration_get` / `delay_get`** (`src/interpolation.rs:415-441`) — `duration_get`
   computed its `min_start` reduction via `.max()` seeded at `0.0` (returning the latest per-element
   delay instead of the earliest); `delay_get` seeded its `.min()` reduction at `0.0` instead of
   `f64::MAX` (always returning `0.0` whenever every real delay was positive). Reproducer:
   `tests/interpolation_test.rs::test_tween_array_duration_and_delay_get`.
2. **`Sequencer::delay_get`** (`src/sequencer.rs:269-278`) — seeded at `f64::MAX` (correct) but reduced
   via `.max( min_delay )` instead of `.min( min_delay )`, so it always returned `f64::MAX`, collapsing
   `progress()` to `0.0` regardless of actual elapsed time. Reproducer:
   `tests/sequencer_test.rs::test_sequencer_delay_get_and_progress_with_delayed_tween`.
3. **`Tween::repeat_handle`** (`src/interpolation.rs:255-278`) — both repeat branches clamped the
   post-wrap elapsed time with `.min( 0.0 )` instead of `.max( 0.0 )`, discarding real leftover progress
   on any repeat that didn't land on an exact duration multiple. Reproducers:
   `tests/interpolation_test.rs::test_tween_infinite_repeat_preserves_overflow_elapsed` and
   `::test_tween_finite_repeat_preserves_overflow_elapsed`.
4. **`Sequence::new`'s Unsorted validation** (`src/sequencer.rs:331-371`) — `last_delay` was declared
   immutable and never reassigned inside the validation loop, so the check always compared against
   `0.0` instead of the previous player's delay, making it dead code for any realistic
   (non-negative-delay) input. Reproducer:
   `tests/sequencer_test.rs::test_sequence_new_rejects_unsorted_players`.

`cargo nextest run -p animation --all-features` confirms all tests pass (29/29), including the 5 new
reproducer tests above.

### API doc table (fixed, 2026-08-09)

`readme.md`'s `## API Reference § Core Components` table described `Sequencer::add()` / `get_value()`,
neither of which exist on the real API — corrected to `insert()` / `get()` / `value_get()`, and added
the missing `Sequence` row (the table previously covered only `Sequencer` / `Tween` / `EasingFunction`).

### Macro-export lint (still deferred — no change from the 2026-08-08 decision below)

Not touched this session. The fix recipe remains empirically verified and ready to re-apply at a future
pickup — re-verify current line numbers in `src/easing/base.rs` / `src/easing/cubic/bezier.rs` first,
since this session's separate `CubicBezier` default-iterations fix (`TASK-041`) shifted line numbers
throughout `bezier.rs`.

Two related findings from the same 2026-08-09 re-audit pass were split into their own tasks rather than
bundled here: `TASK-041` (`CubicBezier` default-iterations bug + `CubicHermite` silent-truncation bug,
both confined to `easing/cubic/`) and `TASK-042` (`Sequencer::value_get` duplication, dead `web-sys`
dependency, `lib.rs` `#[allow(...)]` justification sweep) — both filed and resolved the same session.

The future-incompatibility item below was directly confirmed in an earlier session
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
- **[2026-08-09]** `PARTIAL_FIX` — Re-audited with exact file:line citations (superseding the original
  filing's "3 logic bugs... unverified/uncited" framing) and fixed the bugs + readme API-table concerns
  in full: 4 confirmed logic-bug locations in `Sequencer`/`Tween` fixed in-place with
  `Fix(TASK-015)`/`Root cause`/`Pitfall` source comments and 5 new `bug_reproducer(TASK-015)` tests
  (`cargo nextest run -p animation --all-features`: 29/29 passing); `readme.md`'s Core Components table
  corrected to the real API (`insert()`/`get()`/`value_get()`, added missing `Sequence` row). The
  macro-export lint concern is unchanged — still deferred per the 2026-08-08 decision above, not part of
  this session's authorized scope. Two related findings from the same re-audit pass were split into
  their own tasks rather than bundled here: `TASK-041` and `TASK-042` (see Goal for scope) — both filed
  and resolved the same session. Verification performed as a Tier 2 Dual-Role Self-Check (same session,
  no independent dispatch) per `governance/maav.rulebook.md § MAAV : Verification Tier Selection`'s
  default — not an independent PROC16-style acceptance pass. Task remains open (state unchanged)
  pending the deferred macro-export lint concern.
- **[2026-08-09]** `ENV_NOTE` — During final-verification re-checks (session-level MAAV Tier 2 adversarial
  pass), discovered that `module/min/minwebgl/src/texture/d2.rs` has a pre-existing, uncommitted edit
  (present in `git status`/`git diff` before this session started — not introduced by any work under
  `TASK-015`/`TASK-041`/`TASK-042`) that fails to compile: `get_image_data` is called with `f64` args
  against a `web-sys 0.3.104` signature that expects the old `i32` args (`error[E0308]`). `minwebgl` is a
  mandatory (non-optional) dependency of `animation`, so this blocks any fresh compile of `animation`,
  `renderer`, `tiles_tools`, and `scene_script`. It was silently masked by a stale build-cache artifact
  for narrowly-scoped builds until this session's mandated full-workspace `will .test level::3` run
  forced a cache-invalidating rebuild that exposed it uniformly — re-running the exact `-p animation`
  and `-p renderer` nextest commands that had earlier passed (dated logs `-0018_longrun.log`,
  `-0019_longrun.log`) now reproduces the same `minwebgl` compile failure. The animation-crate fixes
  documented above and in `TASK-041`/`TASK-042` were genuinely verified passing at the time each log was
  captured; they are simply not freshly re-runnable in the current repo state until `minwebgl`'s issue
  is resolved. Out of scope for this task at time of discovery — flagged rather than fixed inline,
  pending explicit user authorization (see follow-up entry below).
- **[2026-08-09]** `ENV_NOTE` (follow-up) — User explicitly authorized fixing the `minwebgl` regression
  above and continuing until the full workspace reaches consistency. Applied: reverted
  `module/min/minwebgl/src/texture/d2.rs`'s `get_image_data` call from `f64` args back to `i32` args,
  matching the confirmed `web-sys 0.3.104` signature and the original pre-regression code. Re-running
  `-p animation -p renderer -p tiles_tools -p scene_script --all-features` nextest afterward passed
  338/338. During the same consistency sweep, a fresh full-workspace `will .test level::3` run surfaced
  7 genuine `clippy::manual_is_multiple_of` violations (all on unsigned-integer sites) across
  `event_system_demo`, `renderer/tests/skeleton_tests.rs`, `renderer/.../wide_outline.rs` (×2),
  `stealth_game`, and `tilemap_renderer/.../svg.rs` (×2) — fixed by converting to `.is_multiple_of()`.
  A broader pattern-matched sweep for the same `% N == 0` shape initially over-applied this conversion to
  9 additional sites that use signed (`i32`) operands, which do not support `.is_multiple_of()` in
  `rustc 1.97.1`; these 9 were caught by the next compile and reverted to their original (already
  clippy-clean) form. A separate, unrelated Cargo manifest error surfaced once
  (`feature 'webgpu' includes 'dep:minwebgpu', but 'minwebgpu' is not listed as a dependency` in
  `module/helper/renderer/Cargo.toml`) — investigation found the manifest already correctly declares
  `minwebgpu` under `[dependencies]`; a clean `cargo metadata --all-features` and the subsequent full
  gate pass confirm this was a transient resolution error (likely from overlapping concurrent cargo
  invocations during this session), not a real defect — no source change was made or needed for it.
  Final state: `will .test level::3` (nextest + doctests + clippy, `--all-features`, `-D warnings`)
  passed 97/97 crates, 0 failed, across the full workspace.
