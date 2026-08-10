# Fix primitive_generation doc-contradicting silent failure and NaN-producing precondition gap

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** 2026-08-10
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
