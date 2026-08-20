# 370: Register lottie_surface_rendering from_angle_y fix (closes BUG-311, split of task 360)

## Execution State

- **id:** 370
- **title:** Register lottie_surface_rendering from_angle_y fix (closes BUG-311, split of task 360)
- **state:** 🔬 (Verifying)
- **open:** true
- **in_motion:** true
- **round:** 1
- **closes:** BUG-311
- **filed:** 2026-08-18 17:47:14
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **executor_type:** any
- **unit_type:** module (`scope` from the crate dir returns `SCOPE_LEVEL=package`, not in tsk.rulebook.md's 5-value enum `yard|repository|workspace|module|dir` -- mapped to the closest valid variant, a single crate/package within a workspace)
- **unit:** lib/yrd_gamedev/cgtools/examples/minwebgl/lottie_surface_rendering
- **actor:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **started_at:** 2026-08-19 22:45:33
- **expires_at:** 2026-08-20 00:45:33
- **related_tasks:** 369 (curve_surface_rendering), 371 (animation_surface_rendering), 372 (ndarray_cg) -- split siblings of cancelled task 360; supersedes task 360's portion for this crate
- **unverified_at:** 2026-08-19 22:37:55
- **unverified_by:** system
- **verifying_at:** 2026-08-19 22:45:33
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/

## MOST Goal

Register `lottie_surface_rendering`'s already-applied, already-verified `from_angle_y`
degrees/radians fix (this crate's slice of BUG-311, `task/bug/verified/311_from_angle_y_called_with_raw_degrees_not_radians.md`,
Medium severity, 🎯 Verified) as a tracked, crate-scoped task. **Motivated** by BUG-311 and by
task 360's own D6 (Crate Scope Unity) FAIL, which found the original multi-crate registration
task illegitimately spanned 4 crates and required an admin `DECOMPOSE_SPLIT` (PROC17) into one
task per crate -- this is that split's `lottie_surface_rendering` slice. **Observable**: the call
site at `src/main.rs:183` states `gl::Quat::from_angle_y( 90.0_f32.to_radians() )`, with a 3-field
`Fix(BUG-311)`/`Root cause`/`Pitfall` source comment immediately above it (verified present, live,
2026-08-18). **Scoped**: exactly one crate, one call site -- no other file or crate touched.
**Testable**: `cargo check -p lottie_surface_rendering --target wasm32-unknown-unknown` -> 0
errors; `grep -c "rotation_set( gl::Quat::from_angle_y( 90.0_f32.to_radians() ) )"
examples/minwebgl/lottie_surface_rendering/src/main.rs` -> 1.

## In Scope

- `examples/minwebgl/lottie_surface_rendering/src/main.rs:177-183` -- the already-applied
  `from_angle_y( 90.0 )` -> `from_angle_y( 90.0_f32.to_radians() )` fix and its
  `Fix(BUG-311)`/`Root cause`/`Pitfall` source comment (verify present; no further edit expected).
- Formal task registration and lifecycle walk (submit, claim-verify, attempt `tsk .verify_pass`)
  for this crate's already-complete fix.

## Out of Scope

- `curve_surface_rendering`, `animation_surface_rendering` (own sibling split tasks -- each
  crate's task registry is self-contained per `tsk.rulebook.md`'s Cross-Crate Deliverable Note;
  no dependency edge to either, since neither appears in this crate's own `Cargo.toml`).
- `module/math/ndarray_cg` and its `test_from_angle_y_rejects_raw_degrees` regression test (own
  sibling split task).
- Any further code change to this crate or to `ndarray_cg::Quat::from_angle_y` itself -- the
  library API is correct and documented as taking radians (BUG-311 Root Cause H1).
- BUG-312 (`character_control`'s own, distinct visible-mesh yaw-halving defect) -- unrelated root
  cause; not this task's concern.
- Re-deriving BUG-311's own MRE or re-running its VERIFY Gate -- already complete and recorded in
  the bug file's Verification Record (2026-08-18, 8/8 PASS).

## Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)

## Delivery Requirements

- All work must strictly adhere to all applicable rulebooks (`kbase .rulebooks`)
- Fix already applied at this crate's one call site: `from_angle_y( 90.0 )` -> `from_angle_y(
  90.0_f32.to_radians() )`, with the 3-field `Fix(BUG-311)`/`Root cause`/`Pitfall` source comment
  in place (re-confirmed live during this task's own filing, 2026-08-18 17:46).
- Green state already confirmed: `cargo check -p lottie_surface_rendering --target
  wasm32-unknown-unknown` clean (re-run live during this task's own filing via `longrun`, exit 0,
  9s combined with sibling crates).
- No refactor needed -- this crate's call site changed only a single literal argument; no
  structural churn.
- Fix documentation already complete at the bug level: BUG-311 carries the 5-section fix
  documentation plus the 3-field source comment convention -- this task does not duplicate it,
  only cross-links via `closes: BUG-311`.
- Task state reaches 🎯 only if this task file's own Readiness Verification Gate genuinely passes
  all 8 dimensions -- D6 (Crate Scope Unity) is expected to PASS this time (exactly one crate,
  `lottie_surface_rendering`, confirmed via `Cargo.toml` inspection), unlike source task 360.
- Independent verification (the post-execution acceptance walk) must pass before this task's
  state advances to ✅ -- reaching 🎯 Verified via this task's own Readiness Verification Gate is
  not sufficient by itself for ✅.
- If the task reaches 🎯: `tsk .verify_pass` is then attempted per standard lifecycle (expected to
  hit this sandbox's known same-actor guard, per project convention and per 7 other sibling
  registration tasks currently at 🔬 Verifying for the same reason -- document rather than
  force/spoof if so).

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---------------|-------------------|-------------------|
| T01 | `grep -c "rotation_set( gl::Quat::from_angle_y( 90.0_f32.to_radians() ) )" examples/minwebgl/lottie_surface_rendering/src/main.rs` | Fixed call site, call-site-anchored pattern | 1 |
| T02 | `grep -c "rotation_set( gl::Quat::from_angle_y( 90.0 ) )" examples/minwebgl/lottie_surface_rendering/src/main.rs` | Naive unfixed pattern must not remain live in code | 0 |
| T03 | `cargo check -p lottie_surface_rendering --target wasm32-unknown-unknown` | Crate compiles for its target platform | 0 errors |
| T04 | `grep -c "Fix(BUG-311)" examples/minwebgl/lottie_surface_rendering/src/main.rs` | Source comment present | >=1 |

## Acceptance Criteria

- `examples/minwebgl/lottie_surface_rendering/src/main.rs` states `from_angle_y(
  90.0_f32.to_radians() )`, not `from_angle_y( 90.0 )`.
- That call site's source comment carries all 3 required fields: `Fix(BUG-311)`, `Root cause`,
  `Pitfall`.
- `cargo check -p lottie_surface_rendering --target wasm32-unknown-unknown` -> 0 errors.
- This task's `closes:` field names `BUG-311`.
- Every Test Matrix row passes.

## Verification

**Execution:** The procedure for walking this section is defined in `§ Acceptance Verification :
Procedure - Execution`. The executor does NOT self-verify -- an independent verifier performs the
walk after the task reaches 🔎 Accepting.

### Checklist

Desired answer for every question is YES.

- [ ] C1 -- Does `examples/minwebgl/lottie_surface_rendering/src/main.rs` state `from_angle_y(
      90.0_f32.to_radians() )` (not `from_angle_y( 90.0 )`)?
- [ ] C2 -- Does that call site's source comment carry all 3 fields: `Fix(BUG-311)`, `Root
      cause`, `Pitfall`?
- [ ] C3 -- Does this task's `closes:` field name `BUG-311`?
- [ ] C4 -- Is `module/math/ndarray_cg/src/` untouched by this task (`git diff --stat` empty for
      that path)?
- [ ] C5 -- Is `examples/minwebgl/curve_surface_rendering/` and
      `examples/minwebgl/animation_surface_rendering/` untouched by this task (`git diff --stat`
      empty for both paths)?

### Invariants

- [ ] I1 -- workspace still builds: `cargo check -p lottie_surface_rendering --target
      wasm32-unknown-unknown` -> 0 errors.

### Anti-faking checks

- [ ] AF1 -- the fix changes only the argument expression (`90.0` -> `90.0_f32.to_radians()`), not
      `from_angle_y`'s own definition or signature -- checked by reading
      `module/math/ndarray_cg/src/quaternion/arithmetics.rs`'s literal diff (expected: empty), not
      just the absence of the old call-site value.

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by user1@w002)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | — | — |
| D4 | Implementation Readiness | — | 🟢 | Adversarial pass specifically checked T02's grep pattern (`rotation_set( gl::Quat::from_angle_y( 90.0 ) )`) against the false-positive risk task 360's own M1/M2 measurements and task 358's D4 finding both documented for this exact comment style — confirmed the call-site-anchored pattern does not match the comment text, so T02's expected `0` is genuine | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | Single crate (`lottie_surface_rendering`, confirmed via `Cargo.toml` `name =` field) — the whole point of this split from task 360 | — |
| D7 | Crate Locality | — | 🟢 | Fix lives at the example binary's own call site, the crate that owns the "clouds" mesh setup — not pushed to an aggregator | — |
| D8 | Crate Single Responsibility | — | 🟢 | Zero code change from this task itself (already applied prior to filing); crate's responsibility unaffected | — |
| **Total** | | — | 🟢 | 0 open | 0/0 |

**Reproduced live during this gate:** `cargo check -p lottie_surface_rendering --target
wasm32-unknown-unknown` (combined `longrun` run with sibling crates, 2026-08-18 17:46:16) → exit
0. `grep -c "rotation_set( gl::Quat::from_angle_y( 90.0_f32.to_radians() ) )"
examples/minwebgl/lottie_surface_rendering/src/main.rs` → 1. `grep -c "Fix(BUG-311)"` same file →
1.

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-18 17:47:14 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | FILED | task created |
| 2026-08-18 17:49:26 | unknown | SUBMIT | structural completeness gate passed |
| 2026-08-18 17:49:32 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-18 17:50 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 370 "user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/"` → blocked: "self-verification forbidden (actor matches filed_by)" (exit 1) — same-actor guard, not a defect; state remains 🔬 Verifying |
| 2026-08-18 23:47:42 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-18 23:49:13 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:37:55 | system | TIMEOUT_2H | 2h exclusivity window expired |
| 2026-08-19 22:45:33 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
| 2026-08-19 22:45:33 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | VERIFY_PASS_ATTEMPTED | `tsk .verify_pass 370` → exit 1, "self-verification forbidden (actor matches filed_by)" — same-actor sandbox guard; not forced/spoofed, left at 🔬 Verifying per standing project convention |

## History

*(append-only -- newest entry last; never edit or remove past entries)*

- **[2026-08-18]** `FILED` -- Task filed via PROC17 (`tsk.rulebook.md § Core Procedures :
  Procedure - Decompose by Crate`) to formally register task 360's `lottie_surface_rendering`
  slice of BUG-311's already-applied, already-verified `from_angle_y` degrees/radians fix
  (`src/main.rs:183`) as a tracked, single-crate task -- one of 4 siblings (369/370/371/372)
  splitting task 360 after its own D6 (Crate Scope Unity) FAIL, per the user's explicit "Yes,
  proceed now" authorization to run DECOMPOSE_SPLIT.
- **[2026-08-18]** `READINESS_GATE_PASS` -- Tier 2 Dual-Role Self-Check, Round 1, 8/8 PASS, no
  fixes needed. Adversarial pass specifically re-checked D4/T02's grep pattern against the
  false-positive risk documented in task 360's own M1/M2 measurements and task 358's D4
  finding, confirming the call-site-anchored pattern does not match the comment text. Re-
  verified live: `cargo check -p lottie_surface_rendering --target wasm32-unknown-unknown` (via
  `longrun`, exit 0); `grep -c "rotation_set( gl::Quat::from_angle_y( 90.0_f32.to_radians() )
  )" examples/minwebgl/lottie_surface_rendering/src/main.rs` → 1; `grep -c "Fix(BUG-311)"` same
  file → 1. `tsk .claim_verify 370` succeeded (❓→🔬, moved to `verifying/`).
- **[2026-08-18]** `EXECUTED` -- No new code edit performed: the described fix
  (`examples/minwebgl/lottie_surface_rendering/src/main.rs:183`, `from_angle_y( 90.0 )` ->
  `from_angle_y( 90.0_f32.to_radians() )`, with its `Fix(BUG-311)`/`Root cause`/`Pitfall`
  source comment) already existed on disk prior to this task's filing, applied during
  BUG-311's own investigation and originally registered under task 360 before its
  DECOMPOSE_SPLIT. This task's own contribution is the formal per-crate tracking registration
  and lifecycle walk, not the code change itself. `tsk .verify_pass 370` blocked by the
  same-actor guard (documented above) — task left at 🔬 Verifying per this sandbox's standing,
  previously-documented limitation (same guard that blocked task 254 and task 358's own
  `.verify_pass`), not a quality defect in this task's own content.
