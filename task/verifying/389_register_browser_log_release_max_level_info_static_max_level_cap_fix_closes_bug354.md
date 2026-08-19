# 389: Register browser_log release_max_level_info STATIC_MAX_LEVEL cap fix closes BUG-354

## Execution State

- **id:** 389
- **title:** Register browser_log release_max_level_info STATIC_MAX_LEVEL cap fix closes BUG-354
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 21:00:26
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/browser_log/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/browser_log
- **closes:** BUG-354
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-18 23:49:14
- **expires_at:** 2026-08-19 01:49:14
- **unverified_at:** 2026-08-18 23:47:43
- **unverified_by:** system
- **verifying_at:** 2026-08-18 23:49:14
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

BUG-354 (`task/bug/verified/354_browser_log_release_max_level_info_silently_caps_logging.md`,
High severity, 🎯 Verified) found `browser_log/Cargo.toml`'s `log` dependency enabled the
`release_max_level_info` feature, which caps `log::STATIC_MAX_LEVEL` at `Info` at COMPILE TIME
in every release-profile build (`cfg(not(debug_assertions))`) — via Cargo feature unification
this cap applies to the ENTIRE dependency graph, silently discarding every `log::debug!`/
`log::trace!` call reached from any crate sharing the build, including a consuming binary's own
unrelated logging, regardless of what `browser_log::log::setup::setup`'s runtime `Config`/
`Level` requested. The fix — removing `release_max_level_info` from the feature list, leaving
only `"std"` — is already applied at `Cargo.toml:58` with a `Fix(BUG-354)` source comment, and
independently confirmed via a new 2-test regression file. Unlike BUG-350/351/352/353,
`browser_log` carries no dependency on `ndarray_cg`/`mdmath_core` (confirmed via `cargo tree -p
browser_log -i ndarray_cg` → no match) and is therefore unaffected by the external workspace
build blocker documented on tasks 385-388 — this task's own live re-run completed cleanly. This
task performs the remaining lifecycle bookkeeping — `tsk.rulebook.md § Core Procedures :
Procedure - Promote Bug to Task` (PROC12) — to formally register that already-complete,
already-verified fix as a tracked task, closing BUG-354. Testable (live-confirmed this task's own
filing, 2026-08-18, `longrun` pid 446955, exit 0): `cd module/helper/browser_log && cargo test -p
browser_log --release --no-fail-fast` → 11 non-doc tests passed (basic_test 2, debug_log_test 1,
panic_hook_test 6, static_max_level_test 2 including the release-only
`static_max_level_is_not_capped_in_release_profile`) + 10 doc-tests passed, 0 failed across every
target.

## In Scope

- `module/helper/browser_log/Cargo.toml` — the already-applied fix (`:58`, `log` dependency's
  `features` list reduced to `[ "std" ]`, `release_max_level_info` removed), with its
  `Fix(BUG-354)` source comment (`:42-57`) — verify present via direct read; no further edit
  expected.
- `module/helper/browser_log/tests/static_max_level_test.rs` (new) — the already-applied
  2-function regression file (`debug_records_reach_the_logger_at_current_build_profile`,
  always-on; `static_max_level_is_not_capped_in_release_profile`,
  `cfg(not(debug_assertions))`-gated) — verify present via direct read; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking
  `task/bug/verified/354_browser_log_release_max_level_info_silently_caps_logging.md`'s header
  back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is filed).

## Out of Scope

- Any further code change to `module/helper/browser_log` — the fix is complete and verified by
  the bug's own Verification Record (8/8 PASS, 2026-08-18).
- Re-running or amending BUG-354's own Verification Record — already run and recorded in the bug
  file; not re-litigated by this task's own Readiness Verification Gate, which checks task-file
  quality, not the underlying fix.
- Auditing other workspace crates for the same `release_max_level_*`/`max_level_*` feature
  family — the bug file's own Generalized Version section already ran
  `grep -rn 'release_max_level_\|max_level_' --include=Cargo.toml .` workspace-wide post-fix (0
  matches outside its own Fix-comment prose); not re-run by this registration task.
- Diagnosing or fixing the external `mdmath_core`/`ndarray_cg` workspace build blocker
  documented on tasks 385-388 — confirmed inapplicable to `browser_log` (no transitive
  dependency), not relevant to this task's own scope.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own Symptom/MRE sections directly
  captured the pre-fix loss (2 of 5 expected log records vanish under `--release`; a standalone
  `/tmp/mre354` crate isolates the cause to the feature choice, not release profile in general).
- Fix already applied at the one site (`Cargo.toml`'s `log` dependency feature list), with the
  required 3-field source comment (`Fix(BUG-354)`/`Root cause`/`Pitfall`).
- Green state already confirmed by the bug file's own Verification Record (2026-08-18) AND
  independently re-confirmed live by this task's own filing (`longrun` pid 446955, exit 0,
  2026-08-18 21:00:13): `cargo test -p browser_log --release --no-fail-fast` → every target
  passes, including the release-only `static_max_level_is_not_capped_in_release_profile`.
- No refactor needed — the fix removes 1 Cargo feature flag, no structural churn, no new public
  surface.
- Fix documentation already complete at the bug level: BUG-354 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention/Pitfall/Generalized Version narrative — this task does not
  duplicate it, only cross-links via `closes: BUG-354`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|--------------------|
| T01 | `cargo test -p browser_log --release --no-fail-fast` | full crate suite, release profile | 11 non-doc + 10 doc-tests passed, 0 failed (live-confirmed this filing) |
| T02 | `cargo test -p browser_log --all-features` (dev profile) | full crate suite, dev profile | passes trivially — dev profile never exercises this defect (bug file's own documented constraint) |
| T03 | `grep -c "Fix(BUG-354)"` in `Cargo.toml` | fix comment present | 1 |
| T04 | `grep -n "release_max_level" module/helper/browser_log/Cargo.toml` | feature genuinely removed | 0 matches outside the Fix-comment prose |

## Acceptance Criteria

- `browser_log/Cargo.toml`'s `log` dependency feature list no longer includes
  `release_max_level_info`
- `static_max_level_test.rs` exists with both the always-on and the
  `cfg(not(debug_assertions))`-gated test, both passing under `--release`
- `task/bug/verified/354_browser_log_release_max_level_info_silently_caps_logging.md`'s header
  states `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row's claim holds against a live run performed during this task's own filing

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does `Cargo.toml`'s `log` dependency read `features = [ "std" ]` (no
  `release_max_level_info`)?
- [ ] C2 — Does `static_max_level_test.rs`'s `cfg(not(debug_assertions))`-gated test assert
  `log::STATIC_MAX_LEVEL == log::LevelFilter::Trace`?
- [ ] C3 — Does the always-on test assert real captured-log-record count via a real
  `log::set_logger`/`log::debug!` call chain (not a hardcoded literal standing in for the call)?
- [ ] C4 — Does `cargo test -p browser_log --release --no-fail-fast` (via `longrun`) pass every
  target, 0 failures?

**Registration correctness**
- [ ] C5 — Does this task's `closes:` field name `BUG-354`?
- [ ] C6 — Does BUG-354's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C7 — No Edit/Write tool call in this task's own execution targeted
  `module/helper/browser_log/src/`, any other workspace crate's `Cargo.toml`, or any
  `module/math/` file — note this repo's working tree carries many pre-existing, unrelated
  uncommitted changes from other concurrent activity, so a blanket repo-wide `git diff --stat`
  is not a meaningful signal here.

### Measurements

- [ ] M1 — `grep -c "Fix(BUG-354)"` in `Cargo.toml` → 1
- [ ] M2 — `grep -n "release_max_level" module/helper/browser_log/Cargo.toml` → 0 matches
  outside the Fix-comment prose lines

### Invariants

- [ ] I1 — `cargo test -p browser_log --release --no-fail-fast` → 0 failures across every target

### Anti-faking checks

- [ ] AF1 — the release-only `static_max_level_is_not_capped_in_release_profile` test reads the
  real `log::STATIC_MAX_LEVEL` constant (not a hardcoded literal standing in for it) — checked by
  reading the test body itself, not just its pass/fail result

## Verification Record

**Gate Round 1** · Tier: 2 · Type: Full · Verdict: OPEN · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
| ---- | ---- | ---- | --- | ------ | ----- |
| D1 | MOST Goal Compliance | — | 🟢 | Confirming: goal states BUG-354, cites PROC12, gives a live-confirmed Testable line. Adversarial: tried to find an overstated claim (e.g. a number not actually observed this session) — the Testable line cites the exact `longrun` pid/exit/timestamp from this task's own filing, not borrowed from the bug file; matches the live log verbatim. | — |
| D2 | Deliverable Verification Completeness | — | 🟢 | Confirming: Verification section carries C1-C7/M1-M2/I1/AF1, each independently checkable. Adversarial: unlike tasks 385-388, ran a genuine live re-verification, not a substitution — `longrun .launch` (pid 446955, exit 0, `-0008_longrun.log`, elapsed 28s) of `cargo test -p browser_log --release --no-fail-fast` confirmed all 5 targets pass (basic_test 2/2, debug_log_test 1/1, panic_hook_test 6/6, static_max_level_test 2/2, 10 doc-tests), matching T01/I1 exactly. Confirmed via `cargo tree -p browser_log -i ndarray_cg` (no match) that this crate carries no transitive dependency on the external `mdmath_core`/`ndarray_cg` blocker documented on tasks 385-388 — genuinely unaffected, not merely untested. C1/C7 verified via direct source read of `Cargo.toml:58` (`features = [ "std" ]`) and the fix comment block `:42-57`. C2-C3 verified via direct read of `static_max_level_test.rs:89-144` — both functions call real `log::set_logger`/`log::debug!`/read the real `log::STATIC_MAX_LEVEL` constant, no hardcoded literal standing in for either. M1/M2 re-confirmed via live grep this gate: `Fix(BUG-354)` count 1, `release_max_level` matches both fall inside the Fix-comment prose (lines 42, 54), 0 outside it. | — |
| D3 | Anti-Cheating Readiness | — | 🟢 | Confirming: AF1 requires reading the actual test body, not trusting pass/fail alone. Adversarial: read `static_max_level_test.rs:132-144` directly this session — `static_max_level_is_not_capped_in_release_profile` asserts `log::STATIC_MAX_LEVEL == log::LevelFilter::Trace` against the real constant, `cfg(not(debug_assertions))`-gated so it only compiles under `--release` — no hardcoded literal found standing in for the constant read. | — |
| D4 | Execution Prerequisites | — | 🟢 | Confirming: `unit_type: module`, `unit: lib/yrd_gamedev/cgtools/module/helper/browser_log`, `closes: BUG-354` all set correctly in Execution State. Adversarial: checked for a mismatched unit path — matches the crate actually holding the fix site. | — |
| D5 | Source-of-Truth Alignment | — | 🟢 | Confirming: no `docs/feature`/`docs/invariant`/`docs/api` instance exists for `browser_log`'s level-cap behavior to conflict with; the crate's own `readme.md` claims ("Configurable log levels for deployment") are restored to true by this fix, not contradicted further. Adversarial: searched for a doc instance that might still describe the compile-time cap as intended behavior — none found; no BLOCKING spec.md/spec/ hygiene violation applies either (dev repo, no spec.md present). | — |
| D6 | Decomposition Fit | — | 🟢 | Confirming: fix spans exactly 1 crate (`browser_log`), 1 manifest line — no multi-crate split warranted. Adversarial: checked whether the bug's own Generalized Version implies other workspace crates need the same fix — no: its own workspace-wide `grep -rn 'release_max_level_\|max_level_' --include=Cargo.toml .` (recorded in the bug file, re-derivable but not re-run here since it's a pure grep with no judgment) already found `browser_log` was the only offender; nothing left to decompose into further tasks. | — |
| D7 | Rulebook Compliance | — | 🟢 | Confirming: no `cargo fmt` invoked, no git command run, Edit used exclusively (task file was `tsk .create`-generated then Edited, never Written after initial creation), all temp artifacts (`-0008_longrun.log`) hyphen-prefixed. Adversarial: scanned this task's own tool-call history for a Write call against a pre-existing file, a non-whitelist git invocation, or a non-hyphenated temp file — none found. | — |
| D8 | Traceability | — | 🟢 | Confirming: `closes: BUG-354` set; bug file backlink to be added immediately after this gate (PROC12 Step 4). Adversarial: verified BUG-354's own file does NOT yet carry a `Fix Task` line (checked via prior Read before this edit) — confirming the backlink write is not a duplicate. | — |
| **Total** | | — | 🟢 | 0 open — full live re-verification obtained, no external blocker applicable to this crate | 0/0 |

Dual-Role Self-Check per `maav.rulebook.md § MAAV : Verification Tier Selection` — Tier 2 default, this session capped at Tier 2 per standing project convention (never escalate).

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 21:00:26 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/browser_log/ | FILED | task created |
| 2026-08-18 21:01:07 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 21:01:07 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/browser_log/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 21:04:00 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/browser_log/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 389 "user1@w002/.../browser_log/"` → exit 1: `tsk: .verify_pass: self-verification forbidden (actor matches filed_by)`. Same-actor guard, documented sandbox constraint — not forced/spoofed. |
| 2026-08-18 23:47:43 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |

## History

- **FILED** (2026-08-18 21:00:26): Task created via `tsk .create`, registering the already-complete BUG-354 fix per PROC12.
- **READINESS_GATE_PASS** (2026-08-18 21:04:00): Tier 2 Dual-Role Self-Check, Gate Round 1, 8/8 dimensions PASS, 0 issues (see Verification Record above). Unlike tasks 385-388, `browser_log` carries no transitive dependency on `ndarray_cg`/`mdmath_core` (confirmed via `cargo tree`), so this gate obtained a full, genuine, live release-profile test re-run (pid 446955, exit 0) rather than substituting bug-file evidence.
- **EXECUTED** (2026-08-18 21:04:00): `tsk .verify_pass` attempted and blocked by same-actor guard, per standard project convention for this sandbox — documented above, not circumvented.
