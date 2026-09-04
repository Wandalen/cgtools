# Fix primitive_generation doc-contradicting silent failure and NaN-producing precondition gap

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
- **unit:** lib/yrd_gamedev/cgtools/module/helper/primitive_generation
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Fix two related `primitive_generation` issues found during the workspace audit (P2 — remaining logic
bugs, Fix-in-place): (1) a function whose doc comment promises an error/validation result instead fails
silently on invalid input; (2) a separate precondition gap that lets degenerate input reach geometry math
and produce `NaN` output rather than being rejected upfront. **Carried forward from the audit triage
plan — exact file/line citations are not re-verified in this filing pass; re-confirm against current
`module/helper/primitive_generation/src/` before touching.** Distinct from task 021 (this crate's
`ufo.rs` dead-code and doc-drift cleanup, a hygiene concern) and from BUG-007/task 008 (this crate's
`csgrs`/`core2` dependency issue) — three separate concerns sharing one crate, keep them separate.

## In Scope

- `module/helper/primitive_generation/src/primitive.rs`: `contours_to_fill_geometry` — triangulation
  failure (`earcutr::earcut` returning `Err`) now returns `None` instead of silently `continue`-ing
  past the failed body, honoring the documented contract
- `module/helper/primitive_generation/src/primitive.rs`: `curve_to_geometry` — added an upfront
  degenerate/coincident-point precondition check (including the wrap-around closing segment) that
  returns `None` instead of letting `.normalize()` produce `NaN`
- New `tests/curve_to_geometry_test.rs` (3 tests) and `tests/contours_to_fill_geometry_test.rs`
  (2 tests) regression coverage

## Out of Scope

- `src/text/ufo.rs` dead-code and doc-drift — separate concern, task 021
- `csgrs`/`core2` dependency issue — separate concern, BUG-007/task 008
- `contours_to_fill_geometry`'s missing `font-processing` feature gate on its `earcutr` usage —
  observed but not one of this task's two named issues; filed separately as task 055

## Verification

### Checklist

- [x] C1 — Is the doc-contradicting silent failure in `contours_to_fill_geometry` genuinely fixed (triangulation `Err` now returns `None` instead of `continue`-ing past the failed body)? Current `src/primitive.rs:299-303`: `let Ok( body_indices ) = earcutr::earcut( &flat_positions, &hole_indices, 2 ) else { return None; };`, directly preceded by a 3-field `Fix(TASK-018)`/`Root cause`/`Pitfall` comment at lines 285-297. Confirmed via direct read; the doc comment's failure contract this fix honors is still present at lines 146-148.
- [x] C2 — Is the NaN-producing precondition gap in `curve_to_geometry` genuinely fixed (degenerate/coincident-point curves rejected before `.normalize()` runs)? Current `src/primitive.rs:62-65`: `if curve.windows( 2 ).any( | pair | pair[ 0 ] == pair[ 1 ] ) || curve.first() == curve.last() { return None; }`, positioned before the first `add_segment` call (line 104) and preceded by a 3-field `Fix(TASK-018)` comment at lines 49-61. Confirmed via direct read.
- [x] C3 — Do both claimed test files still exist with exactly the 5 claimed test functions? `grep -c "#\[ test \]" tests/curve_to_geometry_test.rs` → `3`; `grep -c "#\[ test \]" tests/contours_to_fill_geometry_test.rs` → `2`. All 5 function names match the History's claims exactly: `curve_to_geometry_rejects_single_point_curve`, `curve_to_geometry_rejects_explicitly_closed_curve_with_duplicate_endpoint`, `curve_to_geometry_accepts_non_degenerate_curve_and_produces_finite_positions`, `contours_to_fill_geometry_returns_none_when_triangulation_fails`, `contours_to_fill_geometry_accepts_well_formed_contour`.
- [x] C4 — Do both test files carry the claimed 5-section doc comments (Root Cause / Why Not Caught / Fix Applied / Prevention / Pitfall)? `grep -c "## Root Cause\|## Why Not Caught\|## Fix Applied\|## Prevention\|## Pitfall"` → `5` in each file.
- [x] C5 — Are the History's cited line numbers (`src/primitive.rs:289`; fix comment `272-283`) still accurate today? No — current locations are line `302` (`return None;`) and lines `285-297` (the comment), a ~13-line downward shift. Investigated, not just noted: task 055 (`git show 2be3d2cc`, 2026-08-10 04:56, ~38 min after this task's `9b71cf39` commit) inserted a 10-line `#[cfg(feature = "font-processing")]` gate and comment block directly above `contours_to_fill_geometry`, pushing every subsequent line down. The fix content itself is byte-for-byte unaffected — this is expected citation drift from a later, independent task editing the same function, not a defect in this task.

### Measurements

- [x] M1 — Test function count added under `module/helper/primitive_generation/tests/`: `5` (was: `0` — `git show 9b71cf39^:module/helper/primitive_generation/tests/curve_to_geometry_test.rs` → `fatal: path 'module/helper/primitive_generation/tests/curve_to_geometry_test.rs' exists on disk, but not in '9b71cf39^'`, confirming the directory did not exist before this task's implementing commit).

### Invariants

- [x] I1 — Crate test suite, default features (this task's own fix in `src/primitive.rs` is fully exercised here and is unaffected by the drift in I2/I3 below): `longrun`-launched `cargo nextest run -p primitive_generation` → `Summary [ 0.020s ] 3 tests run: 3 passed, 0 skipped`, exit 0 (the 3 `curve_to_geometry_test.rs` tests; `contours_to_fill_geometry_test.rs` is cleanly excluded by its own Cargo.toml `required-features = ["font-processing"]`, not failed).
- [ ] I2 — Crate test suite, `--all-features` (needed to also run this task's other 2 tests, in `contours_to_fill_geometry_test.rs`): `longrun`-launched `cargo nextest run -p primitive_generation --all-features` → currently FAILS before any test runs: `error[E0639]: cannot create non-exhaustive struct using struct expression` at `src/text/ufo.rs:368` and `src/text/ufo.rs:83`, exit 101. Root cause (see AF3): commit `5f33be66` ("feat: consolidate test infrastructure and refactor module architecture", 2026-08-11 09:30, one day after this task's 2026-08-10 verification) added `#[ non_exhaustive ]` to `mingl::geometry::BoundingBox` (`module/min/mingl/src/geometry.rs:14`). `text/ufo.rs` builds that type via two direct struct-literal expressions (lines 83, 368) that this task never touched — those now fail to compile under any feature combination that compiles `ufo.rs`'s real module (`font-processing`, and therefore `full`/`--all-features`). Not a defect in this task's own fix: task 018 touched only `src/primitive.rs`, never `src/text/ufo.rs`.
- [ ] I3 — Lint cleanliness: `longrun`-launched `cargo clippy -p primitive_generation --all-targets -- -D warnings` (default features) → currently FAILS, but not on this task's code: `error: could not compile browser_log (lib) due to 1 previous error` — `#[ allow( clippy::exhaustive_structs ) ]` at `module/helper/browser_log/src/panic.rs:82` carries no `reason = "..."`, violating the workspace's own `allow_attributes_without_reason = "warn"` lint (root `Cargo.toml:117`) once `-D warnings` promotes it to a hard error. The same commit `5f33be66` last touched that line. `primitive_generation`'s own code contributes zero clippy findings; the failure is inherited transitively before clippy ever reaches this crate.

### Anti-faking checks

- [x] AF1 — Guards against the triangulation-failure branch silently reverting to `continue`: re-run C1's read of `src/primitive.rs` around the `earcutr::earcut` call — the branch must still read `else { return None; }`, never `else { continue; }`.
- [x] AF2 — Guards against the degenerate-curve precondition check being silently removed or narrowed: re-run I1 (`cargo nextest run -p primitive_generation`, default features, unaffected by I2/I3's drift) — must keep showing `3 tests run: 3 passed, 0 skipped`.
- [x] AF3 — Guards against trusting a cached "all green" verification for this crate without re-running it: I2/I3 above are this session's own direct proof that a fully-passing `--all-features` verification recorded one day (2026-08-10, this task's own `Verification Record`) can silently go stale the very next day (2026-08-11) from a completely unrelated commit (`5f33be66`) touching a file (`src/text/ufo.rs`) this task never modified. Before relying on this task's original History claim of "5/5 passed, clean clippy" for anything — e.g. as precedent that the crate currently builds clean — re-run `cargo nextest run -p primitive_generation --all-features` and `cargo clippy -p primitive_generation --all-targets --all-features -- -D warnings` fresh; do not assume a prior day's PASS still holds.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P2 (remaining logic
  bugs) tier, Fix-in-place bucket.

- **[2026-08-10]** `IMPLEMENTED` — Re-verified both citations against current source (both confirmed
  accurate, no drift from filing) and fixed both issues in
  `module/helper/primitive_generation/src/primitive.rs`.

  **Issue 1 (doc-contradicting silent failure)** — `contours_to_fill_geometry` (`src/primitive.rs:147`).
  Its doc comment (`src/primitive.rs:144-146`) promises `Returns ... None if the input contours is
  empty or if the triangulation process fails`, but the per-body triangulation call
  (originally `src/primitive.rs:255-259`) used `let Ok( body_indices ) = earcutr::earcut( .. ) else
  { continue; };` — a triangulation failure was silently swallowed (`continue` to the next body)
  instead of propagating as the documented `None`. When the failing body was the only body, the
  function still returned `Some( PrimitiveData )` with empty `positions`/`indices` rather than `None`.
  Fix: changed `continue` to `return None` (now `src/primitive.rs:289`; 3-field fix comment at
  `src/primitive.rs:272-283`).

  **Issue 2 (precondition gap → NaN)** — `curve_to_geometry` (`src/primitive.rs:38`). It builds every
  stroke segment — including the implicit closing segment from the curve's last point back to its
  first — via `direction = ( end_point - start_point ).normalize()` with no check that the two points
  differ. `F32x2::normalize()` (traced to `mdmath_core::vector::arithmetics::normalize`,
  `module/math/mdmath_core/src/vector/arithmetics.rs:80-92`) divides each component by the vector's
  magnitude with no zero-length guard anywhere in the vector math stack, so a degenerate curve (a
  single point, or any curve whose first and last point already coincide) computed `0.0 / 0.0 = NaN`
  and returned it inside `Some( PrimitiveData )` instead of failing. Fix: added an upfront check
  (`src/primitive.rs:47-63`) that returns `None` when any two consecutive points — including the
  wrap-around pair between the last and first point — are identical, before any segment math runs.

  **Tests added** (new `module/helper/primitive_generation/tests/` directory — the crate had none
  before; both functions are public API, so both live in `tests/` per the workspace
  `rulebook.md`'s Test placement rule):
  - `tests/curve_to_geometry_test.rs`: `curve_to_geometry_rejects_single_point_curve`,
    `curve_to_geometry_rejects_explicitly_closed_curve_with_duplicate_endpoint`,
    `curve_to_geometry_accepts_non_degenerate_curve_and_produces_finite_positions` (regression guard).
  - `tests/contours_to_fill_geometry_test.rs`: `contours_to_fill_geometry_returns_none_when_triangulation_fails`,
    `contours_to_fill_geometry_accepts_well_formed_contour` (regression guard).
  Per TDD, all defect-demonstrating tests were confirmed red against pre-fix code (the two degenerate
  curves produced `Some(..)` containing NaN positions instead of `None`; the NaN-laden 45-point
  contour — needed so `earcutr` 0.5.0's `usehash` z-order path stays enabled and actually reaches its
  `Err` path — produced `Some(..)` with empty attributes instead of `None`), then green after each fix.

  **Verification** — package-scoped, run via `longrun` per policy
  (`dir::.../module/helper/primitive_generation`): `will .test l::3` → `Summary: 4/4 commands passed,
  0 failed` (Local nextest ✅, Workspace nextest ✅, Doc tests ✅, Clippy ✅), exit 0. Independently
  re-confirmed with raw commands: `cargo nextest run -p primitive_generation --all-features` → `5
  tests run: 5 passed, 0 skipped`; `cargo clippy -p primitive_generation --all-targets --all-features
  -- -D warnings` → clean, exit 0; `cargo test -p primitive_generation --doc --all-features` → `3
  passed; 0 failed`.

  **Out of scope, untouched:** `text/ufo.rs` (task 021's dead-code/doc-drift) and the `csgrs`/`core2`
  dependency issue (BUG-007/task 008). Also observed but not fixed, since it's neither of this task's
  two named issues (no doc contradiction, no NaN): `contours_to_fill_geometry` calls
  `earcutr::earcut` unconditionally without being gated behind the `font-processing` feature that
  actually supplies the `earcutr` dependency, so `cargo check -p primitive_generation` fails to
  compile under default features alone — `--all-features` (what `will .test`/nextest/clippy above all
  use) works fine. Independently re-confirmed and filed separately as task 055.

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
| D7 | Crate Locality | — | 🟢 | Both fixed functions are public API (`orphan use` in `mod_interface!`) — confirmed tests correctly live in `tests/`, not in-source | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| B1 | Rulebook Compliance | — | 🟢 | — | — |
| B2 | Test-First Requirement | — | 🟢 | — | — |
| B3 | Evidence of Failure | — | 🟢 | — | — |
| B4 | Proper Fix Only | — | 🟢 | `continue`→`return None` honors the documented contract; upfront zero-length check addresses root cause, not the `NaN` symptom | — |
| B5 | Fix Verification | — | 🟢 | Independently re-ran myself: `longrun`-launched package-scoped `will .test l::3` → exit 0, 4/4; direct `cargo nextest -p primitive_generation --all-features` → 5/5 passed; direct `cargo clippy -p primitive_generation --all-targets --all-features -- -D warnings` → clean | — |
| B6 | Knowledge Preservation | — | 🟢 | 3-field `Fix(TASK-018)`/`Root cause`/`Pitfall` source comments (×2) + 5-section test doc comments (×2) confirmed via direct reads of `git diff` and both new test files | — |
| B7 | Code Cleanliness | — | 🟢 | `git status` scoped to primitive_generation shows only the expected files touched; no stray files | — |
| **Total** | | 🔴 | 🟢 | 0 | 0/0 |

**Aggregate verdict:** PASS — all 15 dimensions clean on both passes, zero Blocking Findings. Verification independently re-executed (`longrun`, direct `cargo nextest`/`clippy`/`check`, full reads of both new test files and the source diff) rather than solely trusted from the implementing subagent's own prose, per this session's Stale Evidence Trust discipline. A genuinely new, distinct defect (missing `font-processing` feature gate on `earcutr` usage) surfaced during investigation and was independently re-confirmed and filed separately as task 055, rather than folded into this task's scope or silently dropped.
