# 385: Register tiles_tools coordinate distance overflow and hex method shadowing fix closes BUG-350

## Execution State

- **id:** 385
- **title:** Register tiles_tools coordinate distance overflow and hex method shadowing fix closes BUG-350
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **filed:** 2026-08-18 20:19:43
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/
- **executor_type:** any
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/helper/tiles_tools
- **closes:** BUG-350
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-20 09:57:37
- **expires_at:** 2026-08-20 11:57:37
- **unverified_at:** 2026-08-20 09:57:12
- **unverified_by:** system
- **verifying_at:** 2026-08-20 09:57:37
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

BUG-350 (`task/bug/verified/350_coordinate_distance_overflow_and_hex_method_shadowing.md`,
Medium severity, 🎯 Verified) found all 6 `distance()` implementations across
`module/helper/tiles_tools`'s 4 coordinate-system files (`hexagonal.rs` — both an inherent `i32`
method and a `Distance` trait `u32` method, `square.rs` — `FourConnected` and `EightConnected`,
`isometric.rs` — `Diamond`, `triangular.rs`) performed their arithmetic directly on raw `i32`
fields, or widened only part of the computation, or narrowed the final result with a bare `as`
cast — reachable via public API alone since every affected coordinate type's fields are `pub`
and its constructor(s) accept the full `i32` range unchecked. 5 of the 6 methods panicked
(`attempt to negate/subtract with overflow`) for coordinates ~2e9 apart or with a component
equal to `i32::MIN`; the 6th (`triangular`) silently wrapped to a wrong value instead of
panicking. A compounding method-shadowing hazard on `hexagonal::Coordinate<Axial,_>` (inherent
`distance` always shadows the `Distance` trait's `distance` in method-call syntax) meant both
methods needed independent fixes rather than deleting either. The fix — widening every operand
to `i64` for the ENTIRE computation at all 6 sites, then narrowing exactly once via
`.clamp( 0, i64::from( TARGET::MAX ) ) as TARGET` — is already applied and independently
confirmed via a new 10-test reproducer file (`tests/coordinates_distance_overflow_test.rs`) and
this task's own live re-run: `cargo nextest run -p tiles_tools --all-features` → 272/272 passed
(includes all 10 reproducer tests), re-confirmed live during this task's own filing (exit 0,
2026-08-18). This task performs the remaining lifecycle bookkeeping —
`tsk.rulebook.md § Core Procedures : Procedure - Promote Bug to Task` (PROC12) — to formally
register that already-complete, already-verified fix as a tracked task, closing BUG-350.
Testable: `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features
2>&1 | grep -q '272 tests run: 272 passed' && echo PASS || echo FAIL` → PASS.

## In Scope

- `module/helper/tiles_tools/src/coordinates/hexagonal.rs` — the already-applied `i64`-widening
  fix on BOTH the inherent `Coordinate<Axial,Orientation>::distance` (`:176-197`) and the
  `Distance` trait impl's `distance` (`:436-462`), each with its own `Fix(BUG-350)`/`Root
  cause`/`Pitfall` source comment — verify present; no further edit expected.
- `module/helper/tiles_tools/src/coordinates/square.rs` — the already-applied fix on
  `Distance for Coordinate<FourConnected>` (`:168-199`) and `Distance for
  Coordinate<EightConnected>` (`:202-229`) — verify present; no further edit expected.
- `module/helper/tiles_tools/src/coordinates/isometric.rs` — the already-applied fix on
  `Distance for Coordinate<Diamond>` (`:261-290`) — verify present; no further edit expected.
- `module/helper/tiles_tools/src/coordinates/triangular.rs` — the already-applied
  saturating-narrow fix on `Distance for Coordinate<Orientation>` (`:211-235`) — verify present;
  no further edit expected.
- The already-applied `tests/coordinates_distance_overflow_test.rs` reproducer (10 tests) —
  verify present and passing; no further edit expected.
- Formal task registration and lifecycle walk (claim, execute-acknowledge, attempt
  `tsk .verify_pass`) for the already-complete fix.
- Linking `task/bug/verified/350_coordinate_distance_overflow_and_hex_method_shadowing.md`'s
  header back to this task via PROC12 Step 4 (performed as a follow-up edit once this file is
  filed).

## Out of Scope

- Any further code change to `module/helper/tiles_tools` — the fix is complete and verified by
  the bug's own two-pass Verification Record (8/8 PASS, 2026-08-18).
- Re-running or amending BUG-350's own Verification Record — already run and recorded in the bug
  file; not re-litigated by this task's own Readiness Verification Gate, which checks task-file
  quality, not the underlying fix.
- Deleting either of hexagonal's two `distance` methods in favor of the other — the bug file's
  own H7/H8 confirmed both are independently load-bearing (inherent: owned-`Self` calling
  convention at `tests/integration/coordinates_tests.rs:33`; trait: any code generic over
  `C: Distance`, e.g. `src/pathfind.rs`'s `astar*` functions) — not re-derived here.
- Any change to `src/pathfind.rs` or other `Distance`-trait consumers — confirmed by the bug
  file's own E12 to already correctly resolve to the (now-fixed) trait method via generic
  bounds; untouched by this fix.

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Failing-first evidence already on record: the bug file's own Symptom section directly captured
  the pre-fix terminal output (8 panics at precise predicted source lines, plus 1 wrong-value
  assertion for `triangular`'s silent wraparound, `3_705_032_704`) via a temporary
  revert-and-rerun of the fix.
- Fix already applied at all 6 sites: every `distance()` implementation now widens to `i64` for
  the entire computation before a single saturating `.clamp( 0, i64::from( TARGET::MAX ) ) as
  TARGET` narrow, with the required 3-field source comment (`Fix(BUG-350)`/`Root cause`/
  `Pitfall`) above each.
- Green state already confirmed, and re-confirmed live during this task's filing: `cargo nextest
  run -p tiles_tools --all-features` → 272 tests run: 272 passed, 0 skipped (via `longrun`, exit
  0, ~1s warm build); reproducer file alone (`cargo test -p tiles_tools --test
  coordinates_distance_overflow_test`) → 10 passed, 0 failed (via `longrun`, exit 0).
- No refactor needed — the fix is a uniform widen-then-saturate pattern applied independently at
  each of the 6 pre-existing method bodies, no structural churn, no new methods, no deletions.
- Fix documentation already complete at the bug level: BUG-350 carries the full Root Cause/Why
  Not Caught/Fix Location/Prevention narrative (including the method-shadowing hazard analysis)
  in its own body — this task does not duplicate it, only cross-links via `closes: BUG-350`.
- Task state reaches 🎯 on this task file's own Readiness Verification Gate; `tsk
  .verify_pass`/`.acceptance_pass` then attempted per standard lifecycle (expected to hit this
  sandbox's known same-actor guard, per project convention — document rather than force/spoof
  if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|--------------------|
| T01 | `cd module/helper/tiles_tools && cargo nextest run -p tiles_tools --all-features` | full crate suite (includes 10 `coordinates_distance_overflow_test` tests) | exit 0, 272/272 passed |
| T02 | `cargo test -p tiles_tools --test coordinates_distance_overflow_test` | reproducer file alone | exit 0, 10 passed, 0 failed |
| T03 | `cargo check -p tiles_tools --all-features` | crate compiles | 0 errors |
| T04 | `grep -c "Fix(BUG-350)"` across the 4 fixed files | fix comment present at every site | hexagonal.rs:2, square.rs:2, isometric.rs:1, triangular.rs:1 (6 total) |
| T05 | `grep -c "clamp( 0, i64::from("` across the 4 fixed files | saturating narrow present at every site | hexagonal.rs:2, square.rs:2, isometric.rs:1, triangular.rs:1 (6 total) |

## Acceptance Criteria

- All 6 `distance()` implementations (hex inherent + trait, square `FourConnected` +
  `EightConnected`, isometric `Diamond`, triangular) widen their entire computation to `i64` and
  narrow exactly once via a saturating `.clamp( 0, i64::from( TARGET::MAX ) ) as TARGET`
- Each of the 6 fix sites carries a `Fix(BUG-350)` source comment with `Root cause` and
  `Pitfall` fields
- `tests/coordinates_distance_overflow_test.rs` exists with 10 tests and all pass
- Neither of hexagonal's two `distance` methods was deleted — both retained and independently
  fixed
- `task/bug/verified/350_coordinate_distance_overflow_and_hex_method_shadowing.md`'s header
  states `**Fix Task:**` pointing at this task, added by PROC12 Step 4 after filing
- Every Test Matrix row passes

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance
Verification : Procedure - Execution`. The executor does NOT self-verify — an independent
verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

**Fix correctness**
- [ ] C1 — Does every one of the 6 `distance()` implementations compute its ENTIRE arithmetic
  (including hex's negations, not only its subtractions) in `i64` before narrowing?
- [ ] C2 — Does every one of the 6 fixed methods narrow its final result via `.clamp( 0,
  i64::from( TARGET::MAX ) ) as TARGET` rather than a bare `as` cast?
- [ ] C3 — Does each of the 6 fix sites carry a `Fix(BUG-350)` source comment with `Root cause`
  and `Pitfall` fields?
- [ ] C4 — Does `cargo test -p tiles_tools --test coordinates_distance_overflow_test` (via
  `longrun`) pass all 10 tests?
- [ ] C5 — Does `cargo nextest run -p tiles_tools --all-features` (via `longrun`) pass 272/272?
- [ ] C6 — Does `cargo check -p tiles_tools --all-features` succeed with 0 errors?
- [ ] C7 — Are both of hexagonal's `distance` methods (inherent and trait) still present and
  independently callable (inherent via method-call syntax, trait via UFCS/generic bounds)?

**Registration correctness**
- [ ] C8 — Does this task's `closes:` field name `BUG-350`?
- [ ] C9 — Does BUG-350's own header carry a `**Fix Task:**` line pointing back at this task's
  ID?

**Out of Scope confirmation**
- [ ] C10 — No Edit/Write tool call in this task's own execution targeted any of the 4
  `module/helper/tiles_tools/src/coordinates/*.rs` files or `src/pathfind.rs` (the fix content
  matches what BUG-350's own already-completed fix applied; this task made no further source
  edit — note this repo's working tree carries many pre-existing, unrelated uncommitted changes
  from other concurrent activity, so a blanket repo-wide `git diff --stat` is not a meaningful
  signal here).

### Measurements

- [ ] M1 — `grep -c "Fix(BUG-350)"` across `src/coordinates/{hexagonal,square,isometric,triangular}.rs`
  → 2, 2, 1, 1 (6 total)
- [ ] M2 — `grep -c "clamp( 0, i64::from("` across the same 4 files → 2, 2, 1, 1 (6 total)
- [ ] M3 — `grep -n "fn distance" src/coordinates/*.rs | wc -l` → 6

### Invariants

- [ ] I1 — `cargo nextest run -p tiles_tools --all-features` → 0 failures
- [ ] I2 — `cargo check -p tiles_tools --all-features` → 0 errors

### Anti-faking checks

- [ ] AF1 — the reproducer tests actually construct coordinates via each type's real public
  constructor and call the real `.distance()`/`Distance::distance()` methods (not hardcoded
  expected-value literals standing in for the calls) — checked by reading the test bodies
  themselves, not just their pass/fail result

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | Adversarial pass confirmed In/Out Scope enumerate all 4 touched files and all 6 fix sites, matching the bug file's own `## Refs: src/` exactly; no scope creep into `src/pathfind.rs` or hex method deletion (both explicitly named Out of Scope, matching the bug's own H7/H8 decision). | — |
| D2 | MOST Goal Quality | — | 🟢 | Confirming pass: this task's own filing-time evidence (2026-08-18 20:19:19 reproducer-only 10/10, and 20:20:05 full-crate 272/272, both via `longrun`, both BEFORE any external interference) directly backs the MOST Goal's Testable claim. Adversarial pass attempted a FRESH live re-run during this gate and hit an external blocker: `cargo nextest run -p tiles_tools --all-features` (and, retried, without `--all-features`) now fails with `error[E0433]: cannot find general in mdmath_core` inside the unrelated transitive dependency `ndarray_cg` (cascading into ~150 unresolved `crate::*` imports). Root-caused via mtime: `module/math/mdmath_core/src/lib.rs` was modified 2026-08-18 20:38:31 — 18 minutes after this task's own clean 20:20:05 run — and its `mod_interface!` block no longer declares a `general` layer (current layers: `approx`/`index`/`float`/`nd`/`plain`/`traits`/`vector`; `float`'s doc comment, "Describe general floats and operations on them," strongly suggests `general` was renamed to `float`), while `ndarray_cg/src/general.rs` (mtime 2026-08-08, untouched) still does `reuse ::mdmath_core::general;`. Bounded poll (6 attempts, ~90s, via `cargo check -p ndarray_cg`) confirmed this is a persistent, not transiently-flickering, broken state — attributable to unrelated concurrent activity on `mdmath_core`/`ndarray_cg`, not this task's own crate (`tiles_tools`'s `src/coordinates/*.rs` is untouched by and unrelated to that refactor). This gate's PASS rests on the task's own pre-existing clean evidence (both runs predate the breaking change) plus direct source re-reads below, not on a fresh full-suite run, which is currently blocked for reasons entirely outside this task's or BUG-350's scope. | — |
| D3 | Cross-Reference Integrity | — | 🟢 | `grep -c "Fix(BUG-350)"` across the 4 fixed files (live, this gate): hexagonal.rs:2, square.rs:2, isometric.rs:1, triangular.rs:1 = 6, matching `## Refs: src/`. `grep -n "fn distance" src/coordinates/*.rs \| wc -l` → 6. `grep -c "clamp( 0, i64::from("` across the same 4 files (adversarial pass corrected the pattern to match this project's space-inside-parens codestyle after an initial no-space grep false-negatived on all 4 files) → 2, 2, 1, 1 = 6. All match. | — |
| D4 | Root Cause Quality | — | 🟢 | Adversarial pass independently re-read all 6 fixed method bodies in full (not just the `Fix(BUG-350)` comment text) across all 4 files — every site widens to `i64` for the ENTIRE computation (including hex's two negations building `s`/`other_s`) before a single saturating `.clamp( 0, i64::from( TARGET::MAX ) ) as TARGET` narrow — matches the bug file's `## Fix Location` claims exactly, no partial-widening regression found. | — |
| D5 | Execution Scope | — | 🟢 | `unit_type: module` / `unit: lib/yrd_gamedev/cgtools/module/helper/tiles_tools` matches the actual crate path; `-p tiles_tools` resolves to this package. | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`tiles_tools`) throughout In Scope/Out of Scope — all 4 touched files live under `module/helper/tiles_tools/src/coordinates/`; no second-crate reference. | — |
| D7 | Crate Locality | — | 🟢 | Confirmed via live read that all 4 fixed files physically live under `module/helper/tiles_tools/src/coordinates/` — matches the `unit` field. | — |
| D8 | Crate Single Responsibility | — | 🟢 | Fix stays within `tiles_tools`'s existing coordinate-math responsibility; no entanglement with `src/pathfind.rs` or other `Distance`-trait consumers (confirmed untouched). | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced (this task's own filing-time run, before external interference):** `cd
module/helper/tiles_tools && cargo test -p tiles_tools --test coordinates_distance_overflow_test`
(via `longrun`) → 10 passed, 0 failed, exit 0, 2026-08-18 20:19:19. `cargo nextest run -p
tiles_tools --all-features` (via `longrun`) → 272 tests run: 272 passed, 0 skipped, exit 0,
2026-08-18 20:20:05. `grep -c "Fix(BUG-350)"` / `grep -c "clamp( 0, i64::from("` across all 4
fixed files both → 2/2/1/1 (6 total), live, this gate. **External blocker (informational, not a
task or fix defect):** a fresh re-run attempted during this gate (20:44-20:46) hit
`ndarray_cg`'s currently-broken `reuse ::mdmath_core::general;` (root-caused above, D2) —
confirmed to also block `scene_script` (a wholly unrelated crate touched by BUG-351, filed
next), so this is a workspace-wide, not tiles_tools-specific, transient external condition.

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 20:19:43 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | FILED | task created |
| 2026-08-18 20:42:41 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 20:42:45 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 385 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/module/helper/tiles_tools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:43 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:55 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:34 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 385` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |
| 2026-08-20 09:57:12 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-20 09:57:37 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-20 10:03:47 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 385` → exit 1, same-actor guard (unchanged). Round 7 fresh-reclaim re-confirmation: BUG-350 still 🎯 Verified, cited fix-file paths still resolve (mechanical drift check clean) |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC12 to formally register BUG-350's
  already-applied, already-verified fix (all 6 `distance()` implementations across
  `module/helper/tiles_tools/src/coordinates/{hexagonal,square,isometric,triangular}.rs` now
  widen to `i64` for their entire computation and narrow via a saturating `.clamp(...)`,
  fixing 5 overflow panics and 1 silent-wraparound corruption, while keeping both of hexagonal's
  independently-load-bearing `distance` methods) as a tracked task, closing the bug.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS.
  Confirming pass relied on this task's own filing-time live evidence (reproducer 10/10, full
  crate 272/272, both clean). Adversarial pass attempted a fresh re-run and discovered an
  external, unrelated blocker: a concurrent in-flight refactor on `mdmath_core` (renamed/removed
  its `general` layer, mtime 20:38:31) broke `ndarray_cg`'s `reuse ::mdmath_core::general;`
  (mtime 2026-08-08, untouched), transitively breaking every workspace crate depending on
  `ndarray_cg` including `tiles_tools`. Root-caused via mtime comparison and a bounded 6-attempt
  poll confirming persistence; documented in full in this task's own Verification Record (D2)
  rather than silently retried or silently ignored. Does not implicate BUG-350's own fix, which
  predates the breaking change and was independently re-confirmed via direct source reads of all
  6 method bodies. `tsk .claim_verify 385` succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix already existed
  on disk prior to this task's filing, applied and verified (bug file's own Verification Record,
  2026-08-18) during BUG-350's own investigation. This task's own contribution is the formal
  tracking registration and lifecycle walk, not the code change itself. `tsk .verify_pass 385`
  blocked by the same-actor guard (documented above) — task left at 🔬 Verifying per this
  sandbox's standing, previously documented limitation, not a quality defect in this task's own
  content.
