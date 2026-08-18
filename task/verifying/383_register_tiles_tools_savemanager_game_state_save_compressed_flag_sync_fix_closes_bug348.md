# 383: Register tiles_tools SaveManager game_state_save compressed flag sync fix (closes BUG-348)

## Execution State

- **id:** 383
- **title:** Register tiles_tools SaveManager game_state_save compressed flag sync fix (closes BUG-348)
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 20:07:26
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **closes:** BUG-348
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/
- **started_at:** 2026-08-18 20:08:10
- **expires_at:** 2026-08-18 22:08:10
- **unverified_at:** 2026-08-18 20:08:10
- **unverified_by:** unknown
- **verifying_at:** 2026-08-18 20:08:10
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/

## MOST Goal

BUG-348 (`task/bug/verified/348_save_manager_meta_compressed_flag_desync.md`, Medium severity,
🎯 Verified) found `module/helper/tiles_tools/src/serialization.rs`'s
`SaveManager::game_state_save` cloning the caller's `SerializableGameState.metadata` and
updating only `size_bytes` before writing the `.meta` sidecar — `metadata.compressed` was
never assigned from `self.serializer.compress` (the field that actually determines whether the
`.save` file's bytes are compressed), so it silently retained whatever value the caller's input
happened to already carry. `game_state_load` never surfaced this because it decides
decompression from `self.serializer.compress` directly, never from the loaded metadata — a
consumer of the `.meta` sidecar alone (e.g. `save_metadata_load`, or an external tool) got a
potentially wrong answer. The fix — assigning `metadata.compressed = self.serializer.compress`
immediately after the existing `size_bytes` write, with the required
`Fix(BUG-348)`/`Root cause`/`Pitfall` 3-field source comment plus a `BUG-348 task/bug/...`
backreference — is already applied and independently confirmed via a new reproducer test
(`test_game_state_save_meta_compressed_flag_matches_actual_compression`,
`tests/serialization_test.rs:245`) proving that saving via a `with_compression(true)`
serializer starting from a `metadata.compressed == false` fixture reloads a `.meta` sidecar
whose `compressed` field now matches the serializer, not the stale input value — the bug
file's own VERIFY Gate, 8/8 PASS, 2026-08-18 (two independent passes, the second one adding
the missing backreference comment), plus a full-suite re-run (272/272 tests, re-confirmed live
during this task's own filing). This task performs the remaining lifecycle bookkeeping —
`tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task` (PROC12) — to formally
register that already-complete, already-verified fix as a tracked task, closing BUG-348.
Testable: `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features
2>&1 | grep -q '272 tests run: 272 passed' && echo PASS || echo FAIL` → PASS.

## In Scope

- `module/helper/tiles_tools/src/serialization.rs` — the already-applied
  `SaveManager::game_state_save` `metadata.compressed = self.serializer.compress` sync fix and
  its `Fix(BUG-348)`/`Root cause`/`Pitfall` source comment plus `BUG-348` backreference —
  verify present; no further edit expected.
- The already-applied
  `tests/serialization_test.rs::test_game_state_save_meta_compressed_flag_matches_actual_compression`
  reproducer — verify present and passing; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/348_save_manager_meta_compressed_flag_desync.md`'s header back to
  this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `module/helper/tiles_tools` — the fix is complete and
  independently verified by the bug's own VERIFY Gate (two independent passes).
- Re-running BUG-348's own VERIFY Gate — already run and recorded in the bug file's
  Verification Record (2026-08-18, 8/8 PASS); not re-litigated by this task's own Readiness
  Verification Gate, which checks task-file quality, not the underlying fix.
- `game_state_load`'s decompression decision path — confirmed by the bug file's own H3/E3 to
  consult `self.serializer.compress` directly, never the metadata's `compressed` field; not
  touched by this fix or this task.
- `SaveMetadata.size_bytes` and other already-correctly-re-derived fields — the bug file's own
  Generalized Version section confirmed `compressed` is the only `SaveMetadata` field with both
  an independent builder and a corresponding `GameStateSerializer` field it should track; not
  re-derived here.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own MRE section directly observed
  the pre-fix desync (`.meta` sidecar reporting `compressed == false` while the `.save` file
  was actually compressed) via a permanent reproducer test run against the pre-fix source —
  this task does not re-derive that evidence.
- Fix already applied: `serialization.rs`'s `game_state_save` assigns `metadata.compressed =
  self.serializer.compress` immediately after the `size_bytes` write, with the required
  3-field source comment plus backreference.
- Green state already confirmed, and re-confirmed live during this task's filing: `cargo
  nextest run -p tiles_tools --all-features` → 272 tests run: 272 passed, 0 skipped (via
  `longrun`, exit 0, ~1s warm build).
- No refactor needed — the fix is a single-line field assignment plus a comment, no structural
  churn.
- Fix documentation already complete at the bug level: BUG-348 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention narrative in its own body — this task does not duplicate
  it, only cross-links via `closes: BUG-348`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features` | `serialization_test::test_game_state_save_meta_compressed_flag_matches_actual_compression` (bug_reproducer) | exit 0, 272/272 passed |
| T02 | `SaveManager` with `GameStateSerializer::new().with_compression(true)`, `game_state.metadata.compressed == false` (fixture default), `game_state_save(...)` then `save_metadata_load(...)` | fixed `game_state_save` | reloaded `.meta`'s `compressed == true` |
| T03 | `cargo check -p tiles_tools --all-features` | crate compiles | 0 errors |
| T04 | `grep -n "metadata.compressed = self.serializer.compress"` | sync assignment present in `game_state_save` | 1 match |

## Acceptance Criteria

- `module/helper/tiles_tools/src/serialization.rs`'s `game_state_save` assigns
  `metadata.compressed = self.serializer.compress` before writing the `.meta` sidecar
- The fix's source comment carries all 3 required fields: `Fix(BUG-348)`, `Root cause`,
  `Pitfall`, plus a `BUG-348` backreference
- `serialization_test::test_game_state_save_meta_compressed_flag_matches_actual_compression`
  exists and passes
- `game_state_load`'s decompression decision path (`self.serializer.compress`) remains
  unmodified
- `task/bug/verified/348_save_manager_meta_compressed_flag_desync.md`'s header states
  `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `game_state_save` in `serialization.rs` assign `metadata.compressed =
  self.serializer.compress`?
- [ ] C2 — Does the fix's source comment carry `Fix(BUG-348)`, `Root cause`, `Pitfall`, and a
  `BUG-348` backreference?
- [ ] C3 — Does `cargo nextest run -p tiles_tools --all-features` (via `longrun`) pass
  `serialization_test::test_game_state_save_meta_compressed_flag_matches_actual_compression`?
- [ ] C4 — Does `cargo check -p tiles_tools --all-features` succeed with 0 errors?
- [ ] C5 — Does `grep -n "metadata.compressed = self.serializer.compress"
  module/helper/tiles_tools/src/serialization.rs` return exactly 1 match?

**Registration correctness**
- [ ] C6 — Does this task's `closes:` field name `BUG-348`?
- [ ] C7 — Does BUG-348's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C8 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/tiles_tools/src/serialization.rs` (the fix content matches what BUG-348's own
  already-completed fix applied; this task made no further source edit to it — note this
  repo's working tree carries many pre-existing, unrelated uncommitted changes from other
  concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful signal
  here).

### Measurements

- [ ] M1 — `grep -c "metadata.compressed = self.serializer.compress"
  module/helper/tiles_tools/src/serialization.rs` → 1

### Invariants

- [ ] I1 — `cargo nextest run -p tiles_tools --all-features` → 0 failures
- [ ] I2 — `cargo check -p tiles_tools --all-features` → 0 errors

### Anti-faking checks

- [ ] AF1 — the reproducer test actually constructs a `SaveManager`/`GameStateSerializer` with
  `with_compression(true)`, calls the real `game_state_save` and `save_metadata_load` (not a
  hardcoded expected-value literal standing in for the call) — checked by reading the test body
  itself, not just its pass/fail result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass re-checked the Tasks Index omission from In Scope — consistent, established precedent across every registration task this batch. | — |
| D2 | MOST Goal Quality | — | 🟢 | Adversarial pass re-ran the Testable line's command live: `cargo nextest run -p tiles_tools --all-features` via `longrun` → `272 tests run: 272 passed, 0 skipped`, exit 0 — claim holds exactly. | — |
| D3 | Value / YAGNI | — | 🟢 | Adversarial pass scanned Delivery Requirements for scope creep — none found. | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial pass independently read `game_state_save`'s fix line and the reproducer test's full live body — the fix (`metadata.compressed = self.serializer.compress` at line 597) and the test's exact scenario (`with_compression(true)`, fixture starting `compressed == false`, `game_state_save` then `save_metadata_load`) both match this task's T02/AF1 claims exactly. | — |
| D5 | Execution Scope | — | 🟢 | Adversarial pass confirmed `unit_type: module` / `unit: lib/yrd_gamedev/cgtools/module/helper/tiles_tools` matches the actual crate path and package name (`-p tiles_tools` ran successfully). | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`tiles_tools`) throughout In Scope/Out of Scope — no second-crate reference found. | — |
| D7 | Crate Locality | — | 🟢 | Confirmed via live read that `serialization.rs` physically lives under `module/helper/tiles_tools/src/` — matches the `unit` field. | — |
| D8 | Crate Single Responsibility | — | 🟢 | Confirmed `game_state_load`'s decompression path (consults `self.serializer.compress` only, never `metadata.compressed`) is untouched by this fix (bug file's own H3/E3, re-confirmed by this task's Out of Scope) — no entanglement. | — |
| **Total** | | — | 🟢 | 0 open | — |

**Reproduced live during this gate:** `cd module/helper/tiles_tools && cargo nextest run -p
tiles_tools --all-features` (via `longrun`) → 272 tests run: 272 passed, 0 skipped, exit 0, ~1s
(warm build). `grep -c "metadata.compressed = self.serializer.compress" src/serialization.rs`
→ 1. Reproducer test body (`tests/serialization_test.rs:245`) read live: constructs a real
`SaveManager` with `GameStateSerializer::new().with_compression(true)`, asserts the fixture
starts `compressed == false`, calls the real `game_state_save`/`save_metadata_load`, asserts
the reloaded metadata's `compressed` is now `true` — matches T02/AF1 exactly. All
Verification-section grep patterns confirmed correct as originally written — no rewording
needed this round.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 20:07:26 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | FILED | task created |
| 2026-08-18 20:08:10 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 20:08:10 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 383 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC12 to formally register BUG-348's
  already-applied, already-verified fix (`module/helper/tiles_tools/src/serialization.rs`'s
  `SaveManager::game_state_save` now assigns `metadata.compressed = self.serializer.compress`
  before writing the `.meta` sidecar, fixing the desync between the reported and actual
  compression state) as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Adversarial pass pre-verified the planned grep/measurement live before writing it into the
  Verification section, and independently read both the fix's line and the reproducer test's
  full live body to confirm the MOST Goal/T02/AF1 claims — all confirmed accurate, no
  rewording needed. Full crate suite re-run live via `longrun` (272/272 passed). `tsk
  .claim_verify 383` succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix already existed
  on disk prior to this task's filing, applied and independently verified (bug file's own
  VERIFY Gate, two independent passes, 2026-08-18) during BUG-348's own investigation. This
  task's own contribution is the formal tracking registration and lifecycle walk, not the code
  change itself. `tsk .verify_pass 383` blocked by the same-actor guard (documented above) —
  task left at 🔬 Verifying per this sandbox's standing, previously documented limitation, not a
  quality defect in this task's own content.
