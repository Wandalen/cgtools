# Fix animation crate's Sequencer/Tween bugs, wrong API doc table, and macro-export lint

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/animation
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
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

## Verification

### Checklist

- [x] C1 — Is `[Tween<T>; N]::duration_get`/`delay_get`'s min/max-reduction bug still fixed? Read
  `src/interpolation.rs:425-451`: `duration_get`'s `min_start` is seeded `f64::MAX` and reduced via
  `.min( min_start )` (line 430); `delay_get` seeds `f64::MAX` and reduces via `.min( min_delay )` (line
  447) — both correct (was: max-seeded/min-seeded-at-0.0). `Fix(TASK-015)`/`Root cause`/`Pitfall` comment
  present at lines 417-424 (current lines drifted from the task's originally-cited `415-441`, since that
  range covered the pre-fix code before this 8-line comment was inserted — self-caused shift, not
  external drift). Reproducer `tests/interpolation_test.rs::test_tween_array_duration_and_delay_get`
  present at line 205, passing (see I1).
- [x] C2 — Is `Sequencer::delay_get`'s reduction direction still fixed? Read `src/sequencer.rs:266-275`:
  seeded `f64::MAX`, reduces via `.min( min_delay )` (line 271) — correct (was `.max`). `Fix(TASK-015)`
  comment at lines 256-264. Reproducer
  `tests/sequencer_test.rs::test_sequencer_delay_get_and_progress_with_delayed_tween` present at line
  219, passing (see I1).
- [x] C3 — Is `Tween::repeat_handle`'s post-wrap elapsed-time clamp still fixed in both repeat branches?
  Read `src/interpolation.rs:257-288`: infinite-repeat branch (line 269) and finite-repeat branch (line
  279) both use `.max( 0.0 )` (was `.min`). `Fix(TASK-015)` comment at lines 247-256. Reproducers
  `tests/interpolation_test.rs::test_tween_infinite_repeat_preserves_overflow_elapsed` (161) and
  `::test_tween_finite_repeat_preserves_overflow_elapsed` (172) both present, passing (see I1).
- [x] C4 — Is `Sequence::new`'s Unsorted-validation reassignment still fixed? Read
  `src/sequencer.rs:334-374`: `last_delay = player.delay_get();` reassignment present inside the loop
  (line 355), keeping the `last_delay > player.delay_get()` check live (was dead code, always comparing
  against `0.0`). `Fix(TASK-015)` comment at lines 341-347. Reproducer
  `tests/sequencer_test.rs::test_sequence_new_rejects_unsorted_players` present at line 254, passing (see
  I1).
- [x] C5 — Are exactly 4 `Fix(TASK-015)` source comments present (one per bug)? `grep -rn
  "Fix(TASK-015)" src/` → 4 hits: `sequencer.rs:256`, `sequencer.rs:341`, `interpolation.rs:247`,
  `interpolation.rs:417`.
- [x] C6 — Are exactly 5 `bug_reproducer(TASK-015)`-tagged tests present, each with the full 5-section
  doc comment (Root Cause/Why Not Caught/Fix Applied/Prevention/Pitfall)? `grep -rn
  "bug_reproducer(TASK-015)" tests/` → 4 tag comments (one tag block is shared by the two near-duplicate
  repeat-overflow tests at `interpolation_test.rs:141`, covering both
  `test_tween_infinite_repeat_preserves_overflow_elapsed` and
  `test_tween_finite_repeat_preserves_overflow_elapsed`); read all 4 blocks in full — each has all 5
  sections present.
- [x] C7 — Does `readme.md`'s Core Components table show the real API (not the fictitious
  `add()`/`get_value()`), with the previously-missing `Sequence` row added? Current table
  (`readme.md:113-118`) has 4 rows: `Sequencer` (`insert()`, `get()`), `Sequence` (new row:
  `current_get()`), `Tween` (`value_get()`), `EasingFunction`. Confirmed pre-fix baseline via `git show
  0a6c9cc0:module/helper/animation/readme.md`: only 3 rows (`Sequencer` with `add()`/`get_value()`,
  `Tween`, `EasingFunction`) — no `Sequence` row. `grep -n "get_value\|\.add(\|::add(" readme.md` → 0 hits
  now.
- [x] C8 — Does the macro-export future-incompat concern (resolved 2026-08-10 via git-log archaeology)
  still hold under a fresh, independent re-check? `impl_easing_function` is still defined at crate root
  in `src/lib.rs`, now at lines 23-47 (doc comment 11-22) — drifted from the 2026-08-10-recorded
  `55-76`/`43-54`. Investigated: `git diff 67cea248 HEAD -- src/lib.rs` shows the shift is caused entirely
  by removal of 8 unrelated file-level `#![allow(clippy::...)]`/`#![allow(dead_code)]` lines that used to
  sit above the macro (out-of-scope cleanup from a later session, not a TASK-015 concern) — the macro's
  own body and rationale comment are untouched. `grep -rn "macro_export" src/` → 0 active-attribute hits
  (only 2 doc-comment prose mentions). Re-ran (not just re-trusted) `cargo check -p animation --target
  wasm32-unknown-unknown --lib` fresh via `longrun` → exit 0, 15s; grepped the full log for `warning` (8
  hits, all attributed to the `minwebgl` dependency, 0 attributed to `animation`) and for
  `macro_expanded_macro_exports_accessed_by_absolute_paths` (0 hits). Concern remains genuinely resolved.
- [x] C9 — Does the documented `minwebgl` `get_image_data` regression-detour fix still hold? Read
  `module/min/minwebgl/src/texture/d2.rs:299-367`: the code has evolved past TASK-015's own
  single-signature revert — a later, separate fix (`Fix(BUG-053)`, lines 350-363) replaced it with a
  `#[cfg(web_sys_unstable_apis)]`-gated dual branch (`i32` args when the cfg is on, line 365; `f64` args
  when off, line 367), permanently resolving the same root flip-flop TASK-015 had patched narrowly. Not a
  regression — a superseding, more robust fix on the same call site. The underlying "`animation` still
  compiles against its mandatory `minwebgl` dependency" claim is independently reconfirmed by I1's fresh
  passing run (nextest must compile `minwebgl` transitively to run `animation`'s tests).
- [x] C10 — Are the 7 claimed `clippy::manual_is_multiple_of` conversions still in place across all 5
  cited files? 5 of 7 sites confirmed still converted: `examples/tiles_tools/event_system_demo/src/main.rs:225`
  (`.is_multiple_of(3)`), `examples/tiles_tools/stealth_game/src/main.rs:423` (`.is_multiple_of(5)`),
  `module/helper/renderer/tests/skeleton_tests.rs:207` (`.is_multiple_of(2)`),
  `module/helper/tilemap_renderer/src/adapters/svg.rs:1177,1200` (both `.is_multiple_of(2)`). The
  remaining 2 sites (`module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs`, both on
  `u32` operands) are **regressed**: both now read `% 2 == 0` again (lines 374, 441). Root-caused via `git
  show 5f33be66 -- .../wide_outline.rs`: commit `5f33be66` ("feat: consolidate test infrastructure and
  refactor module architecture", 2026-08-11 — unrelated to TASK-015) explicitly reverts both `i.is_multiple_of(
  2 )` → `i % 2 == 0` and `self.num_passes.is_multiple_of( 2 )` → `self.num_passes % 2 == 0`, while
  leaving the other 4 files' conversions untouched despite touching all 5 files in that same commit. A
  genuine regression, not benign drift — but currently **invisible to the crate's own lint gate**:
  `renderer/Cargo.toml` pins `rust-version = "1.75.0"` (added in 2025, long predates TASK-015), and
  clippy's `manual_is_multiple_of` lint is MSRV-gated — even a forced `cargo clippy -p renderer
  --all-targets --all-features -- -W clippy::manual_is_multiple_of` run (via `longrun`) produced zero
  diagnostics against `wide_outline.rs`. Flagged for awareness; out of scope to fix here (not in this
  task's file list).

### Measurements

- [x] M1 — `bug_reproducer(TASK-015)`-covered test count in `tests/interpolation_test.rs` +
  `tests/sequencer_test.rs`: `21` (12+9) (was: `16` (9+7) — confirmed via `git show
  0a6c9cc0:module/helper/animation/tests/interpolation_test.rs` and `...sequencer_test.rs`, `grep -c "#\[
  *test *\]"` on each). Delta `+5` matches the 5 claimed reproducer tests exactly.
- [x] M2 — `readme.md` Core Components table row count: `4` (was: `3` — confirmed via `git show
  0a6c9cc0:module/helper/animation/readme.md`, which had `Sequencer`/`Tween`/`EasingFunction` only, no
  `Sequence` row).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p animation --all-features` → exit 0, 29/29
  passed (via `longrun`).
- [x] I2 — Compiler/lints, deny-warnings (crate-scoped): `cargo clippy -p animation --all-targets
  --all-features -- -D warnings` → **exit 101, NOT clean** — but the failure is not in `animation` or
  anything TASK-015 touched: it fails compiling the transitive dependency `browser_log`
  (`module/helper/browser_log/src/panic.rs:82`, `#[allow(clippy::exhaustive_structs)]` without a `reason
  = "..."` clause, which `-D warnings` promotes to a hard error via
  `clippy::allow_attributes_without_reason`). `cargo tree -p animation --all-features -i browser_log`
  confirms the path: `animation → mingl/minwebgl → browser_log`. Already tracked separately and out of
  scope here: `task/draft/058_workspace_allow_sweep_per_crate.md` (workspace-wide `#[allow]`
  justification sweep).
- [x] I3 — Isolation check for I2 (crate-scoped, no deny-warnings): `cargo clippy -p animation
  --all-targets --all-features` (no `-D warnings`) → exit 0; `animation` itself reports 20 warnings
  (pre-existing pedantic-tier style suggestions, unrelated to any TASK-015 claim), confirming
  `animation`'s own code has no new hard failures — I2's exit 101 is entirely attributable to the
  unrelated `browser_log` blocker.

### Anti-faking checks

- [x] AF1 — Guards against any of the 4 min/max-reduction bugs (duration_get/delay_get array reduction,
  Sequencer::delay_get, Tween::repeat_handle's two branches, Sequence::new's Unsorted check) silently
  recurring: re-run `cargo nextest run -p animation --all-features` (I1) and confirm these 5 tests still
  show PASS: `test_tween_array_duration_and_delay_get`,
  `test_sequencer_delay_get_and_progress_with_delayed_tween`,
  `test_tween_infinite_repeat_preserves_overflow_elapsed`,
  `test_tween_finite_repeat_preserves_overflow_elapsed`, `test_sequence_new_rejects_unsorted_players` —
  each was constructed so only the buggy seed/direction/reassignment fails it (see each test's own Root
  Cause doc comment).
- [x] AF2 — Guards against `readme.md`'s Core Components table drifting from the real API again: re-grep
  the table (`readme.md:113-118`) against each type's real public methods; specifically `grep -n
  "get_value\|\.add(\|::add(" readme.md` must stay at 0 hits (the two fictitious pre-fix method names).
- [x] AF3 — Guards against `#[macro_export]` being reintroduced on `impl_easing_function` (reviving the
  `macro_expanded_macro_exports_accessed_by_absolute_paths` future-incompat lint): `grep -rn
  "macro_export" src/` must stay at 0 active-attribute hits, and `cargo check -p animation --target
  wasm32-unknown-unknown --lib` must stay free of that lint name in its output.

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
- **[2026-08-10]** `VERIFIED_RESOLVED` — Picked up the deferred macro-export lint item. Before editing,
  re-grepped current locations per the task's own line-drift warning, and found drift far larger than
  anticipated: `impl_easing_function` is no longer defined in `src/easing/base.rs` at all.
  `grep -rn 'macro_export\|impl_easing_function' src/` shows it now lives at crate root in
  `src/lib.rs:55-76` (doc comment `:43-54`), already **without** `#[ macro_export ]`, with the doc comment
  (`lib.rs:49-54`) explicitly naming `macro_expanded_macro_exports_accessed_by_absolute_paths` as the
  reason for the relocation. `#[ macro_export ]` has zero active occurrences anywhere in the crate (only 2
  doc-comment-prose mentions, `lib.rs:50,52`). `bezier.rs` no longer has the
  `use crate::{ impl_easing_function, Animatable };` line Goal-section step 4 targeted — its 24
  `impl_easing_function!` invocations (now `bezier.rs:131-161`, drifted from the cited `114-144`) resolve
  via textual macro scope with no `use` needed; `Animatable` is imported separately (`bezier.rs:5`).
  `git log --follow` traced this relocation to commit `67cea248` ("docs: add comprehensive architecture
  and project documentation", 2026-08-09 01:23:50 +0300) — already committed, predating even this same
  day's `UPDATED`/`PARTIAL_FIX` entries above, and `git status`/`git diff` confirm zero uncommitted changes
  in `module/helper/animation/`. The base.rs-targeted recipe's file-level preconditions (macro defined
  inside `mod private` in `base.rs`; `bezier.rs` importing it via that specific absolute-path `use` line)
  no longer exist anywhere in the current tree, so the 4-step recipe was **not** applied — its edit
  anchors don't exist to match against, and forcing an equivalent structure back into `base.rs` would
  duplicate/conflict with the already-working `lib.rs` definition (a compile error) for no benefit over
  the cleaner single-relocation design already in place. No source files were edited this session.
  Independently verified the underlying goal (warning eliminated) holds regardless: both commands run
  package-scoped via `longrun` from `module/helper/animation/` — `will .test l::3` → exit 0, "4/4 commands
  passed, 0 failed", 56s (`-0003_longrun.log`; this level runs under `RUSTFLAGS="-D warnings"`, so a live
  future-incompat warning would have hard-failed it) — and
  `cargo check -p animation --target wasm32-unknown-unknown --lib` → exit 0, 4s (`-0004_longrun.log`),
  output containing only `Checking`/`Finished` lines; grepped explicitly for `warning` and for
  `macro_expanded_macro_exports_accessed_by_absolute_paths`, zero matches for either. This deferred item
  is confirmed resolved — via pre-existing committed code, not a change made this session. `state:` field
  left unchanged per instruction; a separate verification pass covers the `## Verification Record` gate.

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 15/15

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| B1 | Rulebook Compliance | — | 🟢 | — | — |
| B2 | Test-First Requirement | — | 🟢 | Sequencer/Tween bugs TDD-fixed 2026-08-09; macro item needed no code change | — |
| B3 | Evidence of Failure | — | 🟢 | "Already resolved" claim independently confirmed by me via `grep`/`git log`, not just trusted | — |
| B4 | Proper Fix Only | — | 🟢 | Correctly declined to force-reintroduce the old macro-export structure once its preconditions no longer existed | — |
| B5 | Fix Verification | — | 🟢 | Independently confirmed myself: `lib.rs:55` has the macro with no `#[macro_export]`; `git status --porcelain module/helper/animation/` is clean | — |
| B6 | Knowledge Preservation | — | 🟢 | History traces relocation to commit `77cc9b9a`/`67cea248`, cites current line numbers, explains why the old recipe no longer applies | — |
| B7 | Code Cleanliness | — | 🟢 | Zero source diff; task-file edit purely additive | — |
| **Total** | | 🔴 | 🟢 | 0 | 0/0 |

**Aggregate verdict:** PASS — all 15 dimensions clean on both passes, zero Blocking Findings. Verification independently re-executed (`grep`, `git log --follow`, `git status --porcelain`) rather than solely trusted from the implementing subagent's own prose, per this session's Stale Evidence Trust discipline.
