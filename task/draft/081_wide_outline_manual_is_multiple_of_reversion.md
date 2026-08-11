# Reconvert wide_outline.rs's 2 sites back to clippy::manual_is_multiple_of

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/renderer
- **verified_by:** null
- **verification_date:** null
- **blocked_by:** null

## Goal

`module/helper/renderer/src/webgl/post_processing/outline/wide_outline.rs` has 2 sites that were
converted to `clippy::manual_is_multiple_of` form by task 015, then silently reverted back to `% 2
== 0` by an unrelated commit. Confirmed fresh this session, current source:

- Line 374: `else if i % 2 == 0 // Even steps ( 2, 4, ... ) read from FB 1, render to FB 0`
- Line 441: `if self.num_passes % 2 == 0`

Both should read `.is_multiple_of( 2 )` (e.g. `i.is_multiple_of( 2 )` /
`self.num_passes.is_multiple_of( 2 )`), matching the other 5 (of task 015's original 7) sites that
remain correctly converted across the workspace (`event_system_demo`, `stealth_game`,
`renderer/tests/skeleton_tests.rs`, `tilemap_renderer/.../svg.rs` ×2).

Root cause (per task 015's own C10 finding, re-confirmed fresh this session): commit `5f33be66`
("feat: consolidate test infrastructure and refactor module architecture", 2026-08-11 — unrelated
to task 015) explicitly reverts both conversions in this one file, while leaving the other 4 files'
conversions untouched despite touching all 5 files in that same commit.

**Cosmetic/style-only, no behavior change** — `i % 2 == 0` and `i.is_multiple_of( 2 )` are
equivalent for the `u32` operands at both sites. **Currently invisible to this crate's own lint
gate**: `module/helper/renderer/Cargo.toml` pins `rust-version = "1.75.0"` (added in 2025, predates
task 015), and clippy's `manual_is_multiple_of` lint is MSRV-gated — confirmed fresh this session
(`grep "rust-version" module/helper/renderer/Cargo.toml` → `1.75.0`) — so a plain `cargo clippy -p
renderer` run will not catch this regression on its own; task 015 confirmed a forced `-W
clippy::manual_is_multiple_of` override also produces zero diagnostics against this file. This is
purely a style-consistency fix, not a defect with observable runtime impact.

**Related Tasks:** `015` (`task/completed/015_animation_sequencer_bugs_and_api_doc.md`) — this
task's own § Verification C10 first discovered and documented this regression but explicitly left
it unfixed as out of scope for that task's own file list ("Flagged for awareness; out of scope to
fix here").

## History

- **[2026-08-11]** `INVALIDATED` — **Execution attempted; the crate's own lint gate rejects the
  requested conversion, and the evidence overturns this task's premise.** Applied both edits
  exactly as specified (`i.is_multiple_of( 2 )` at 374, `self.num_passes.is_multiple_of( 2 )` at
  441), then ran `cargo clippy -p renderer --no-deps --all-targets --all-features -- -D warnings`
  → exit 101 (`renderer/-0015`): **`incompatible_msrv` fires on both sites** —
  `u32::is_multiple_of` was stabilized in Rust 1.87.0 and `renderer` pins `rust-version =
  "1.75.0"`, so the conversion is an MSRV violation in lib code. This reframes the Goal's
  "silently reverted by an unrelated commit": commit `5f33be66`'s reversion of exactly these 2
  sites was *correct MSRV enforcement*, not regression — and the reason the other 5 of task 015's
  sites survive is that they live in tests/examples (`renderer/tests/skeleton_tests.rs`,
  `event_system_demo`, `stealth_game`, `tilemap_renderer` svg.rs), where clippy's
  `incompatible_msrv` deliberately does not fire (test code is exempt) or no comparable
  `rust-version` pin applies. Both edits reverted; gate re-run green (`renderer/-0016`, exit 0);
  both task sites restored to their pre-task form (`grep -c '% 2'` = 2, `is_multiple_of` = 0 hits
  — verified again post-revert). The file's only remaining working-tree diff vs HEAD is an
  unrelated, pre-existing doc fix on `WideOutlinePass::new` (copy-paste name `NarrowOutlinePass`
  corrected + `# Errors`/`# Panics` sections) from the concurrent sweep effort — untouched by this
  task's execution and revert. **Resolution paths,
  user's call:** (a) close this task as invalid — `% 2 == 0` is the only MSRV-1.75-valid form in
  this crate's lib code, and the current source is already correct; or (b) re-scope it to "raise
  `renderer`'s `rust-version` to ≥ 1.87" (a consumer-facing policy change this task's executor
  won't make unilaterally), after which the 2-site conversion becomes legal and trivial.
- **[2026-08-11]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) during this session's TA106
  out-of-scope-findings triage. Classified via `tsk.rulebook.md § Task File : Deduplication Search`
  as Case E (closed task 015 exists and already names this exact regression, but its own scope —
  the animation sequencer bugs/API doc work — is fully resolved and explicitly excludes fixing
  this; this task's scope — actually applying the 2-site reversion — is a distinct, not-yet-tracked
  unit of work). Cross-linked to 015 via `**Related Tasks:**`. Confirmed via `grep -rl
  "manual_is_multiple_of" task/draft/ task/bug/` that no other tracker already covers this.
