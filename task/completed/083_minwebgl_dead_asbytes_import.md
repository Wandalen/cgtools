# Delete minwebgl's 3 dead unused-import sites (geometry.rs, buffer.rs, ubo.rs)

## Execution State

- **Executor Type:** any
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **actor:** null
- **started_at:** null
- **expires_at:** null
- **round:** 1
- **state:** ✅ (Completed)
- **closes:** null
- **repo_identity:** self
- **unit_type:** module
- **unit:** lib/yrd_gamedev/cgtools/module/min/minwebgl
- **verified_by:** self (Tier 2 Dual-Role Self-Check, acceptance verification)
- **verification_date:** 2026-08-11
- **blocked_by:** null
- **unverified_at:** 2026-08-11 18:25:30
- **unverified_by:** unknown
- **in_motion:** false
- **verifying_at:** 2026-08-11 18:25:50
- **verifying_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/
- **priority:** 0

## Goal

`module/min/minwebgl` currently emits 5 `unused_imports` warnings across 3 files — confirmed fresh
this session via `cargo check -p primitive_generation --features font-processing` (which compiles
`minwebgl` as a transitive dependency):

1. **`src/geometry.rs:4`** — `AsBytes` imported, zero uses anywhere in the file (`grep -n AsBytes
   module/min/minwebgl/src/geometry.rs` → only the import line itself). Already documented: task
   062's own § Verification I3/AF1 — the deleted switch task 062's own change removed was
   `AsBytes`'s only consumer, and the import was not removed alongside it. Task 062 explicitly
   reported this as unresolved drift rather than fixing it ("outside this verification pass's edit
   scope").
2. **`src/buffer.rs:4`** — bare `AsBytes` imported, but the file only ever uses the fully-qualified
   `mem::AsBytes` (line 43: `Data : mem::AsBytes + ?Sized`) — the unqualified import is redundant,
   not orphaned-by-deletion like site 1, but genuinely unused all the same. Newly confirmed this
   session; not previously documented anywhere in `task/`.
3. **`src/ubo.rs:3`** — same redundant-import pattern as site 2 (file uses `mem::AsBytes` at line
   16, not the bare import), plus 2 more dead imports on the same `use` line: `VariantIterator` and
   `IntoEnumIterator`, both zero-use in the file. Newly confirmed this session; not previously
   documented anywhere in `task/`.

All 5 are compile-time-provable dead imports (`unused_imports` lint, not a judgment call) — deleting
them is a purely mechanical, zero-behavior-change cleanup. Scoped to one crate (`minwebgl`) as a
single unit of work per `tsk.rulebook.md`'s Crate Scope Unity/Crate Locality principles, rather than
filing 3 near-identical micro-tasks for the same defect class.

**Related Tasks:** `062` (`task/completed/062_minwebgl_marker_resolution.md`) — its own I3/AF1
finding first identified site 1 (`geometry.rs`) but explicitly left it unfixed as out of that
verification pass's scope. AF1 there already specifies the exact re-check: `grep -n AsBytes
module/min/minwebgl/src/geometry.rs` must show more than 1 hit before that specific finding can be
marked resolved — satisfied automatically once this task deletes the dead import (0 hits after,
which is the correct "resolved by deletion" outcome, distinct from AF1's "resolved by adding a real
use" framing; either ending removes the drift).

## In Scope

-   `module/min/minwebgl/src/geometry.rs`, `src/buffer.rs`, `src/ubo.rs` — deleting the 5 dead-import
    tokens across their 3 `use` statements (site 1: `AsBytes` at `geometry.rs:4`; site 2: bare
    `AsBytes` at `buffer.rs:4`; site 3: `AsBytes`, `VariantIterator`, `IntoEnumIterator` at
    `ubo.rs:3`), while preserving the 2 legitimate fully-qualified `mem::AsBytes` uses

## Out of Scope

-   Any other `minwebgl` file or import site not named above — single defect class, 3-file scope
-   Any behavioral change — compiler-provable dead-import deletion only; no `pub` API, logic, or
    runtime behavior is touched
-   The 2 legitimate fully-qualified uses (`buffer.rs:43`, `ubo.rs:16`, `Data : mem::AsBytes +
    ?Sized`) — these must remain untouched; deleting them would break compilation
-   Any other crate — `minwebgl` only

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any), not by this
section.

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)
-   Mechanical, compiler-provable dead-import deletion only — no new logic, no test code produced;
    Test Matrix is not applicable (nothing behavioral to assert), correctness is instead captured by
    the `-D warnings` compiler gate recorded as an Invariant below
-   Independent verification passes per `§ Acceptance Verification : Procedure - Execution` before
    task state is updated to ✅
-   Task state updated to ✅ only upon verification pass; file moved to `task/completed/`

## Acceptance Criteria

-   `module/min/minwebgl/src/geometry.rs`'s `use` statement (line 4) no longer imports `AsBytes`
-   `module/min/minwebgl/src/buffer.rs`'s `use` statement (line 4) no longer imports bare `AsBytes`
    (the fully-qualified `mem::AsBytes` use at line 43 is untouched)
-   `module/min/minwebgl/src/ubo.rs`'s `use` statement (line 3) no longer imports `AsBytes`,
    `VariantIterator`, or `IntoEnumIterator` (the fully-qualified `mem::AsBytes` use at line 16 is
    untouched)
-   `cargo clippy -p minwebgl --no-deps --all-targets --all-features -- -D warnings` exits 0
-   `git diff --stat -- module/min/minwebgl/` (against the commit that introduced the fix) touches
    only `geometry.rs`, `buffer.rs`, `ubo.rs`

## Verification

**Execution:** The procedure for walking this section is defined in
`§ Acceptance Verification : Procedure - Execution`. The executor does NOT self-verify — an
independent verifier performs the walk after the task reaches 🔎 Accepting.

### Checklist

**Dead-import deletion**
- [x] C1 — Is `geometry.rs`'s `use` statement free of `AsBytes`?
- [x] C2 — Is `buffer.rs`'s `use` statement free of bare `AsBytes`?
- [x] C3 — Is `ubo.rs`'s `use` statement free of `AsBytes`, `VariantIterator`, and
      `IntoEnumIterator`?

**Out of Scope confirmation**
- [x] C4 — Are the 2 legitimate fully-qualified `mem::AsBytes` uses (`buffer.rs:43`, `ubo.rs:16`)
      still present and does the crate still compile?
- [x] C5 — Does `git diff --stat -- module/min/minwebgl/` touch only the 3 named files?

### Measurements

- [x] M1 — `grep -n 'AsBytes\|VariantIterator\|IntoEnumIterator' module/min/minwebgl/src/geometry.rs
      module/min/minwebgl/src/buffer.rs module/min/minwebgl/src/ubo.rs` → exactly 2 hits, both the
      legitimate fully-qualified `mem::AsBytes` uses (was: 5 dead-import warnings per this task's own
      Goal)

### Invariants

- [x] I1 — `cargo clippy -p minwebgl --no-deps --all-targets --all-features -- -D warnings` → exit 0
- [x] I2 — `git diff --stat -- module/min/minwebgl/` (against the commit that introduced the fix)
      shows a real, non-empty diff touching only the 3 named files

### Anti-faking checks

- [x] AF1 — The fix isn't achieved by suppressing the lint (`#[allow(unused_imports)]`) instead of
      actually deleting the dead imports — grep for `allow(unused_imports)` near the 3 sites must
      show none newly added
- [x] AF2 — Task 062's own AF1 re-check contract (`grep -n AsBytes
      module/min/minwebgl/src/geometry.rs` must show more than 1 hit before that finding counts as
      resolved) is satisfied by the stricter "resolved by deletion" outcome — actual result is
      exactly 0 hits

## Journal

| Timestamp           | Actor                | Event | Note         |
|---------------------|----------------------|-------|--------------|
| 2026-08-11 18:25:30 | unknown | SUBMIT | structural completeness gate passed |

## History

- **[2026-08-11]** `RESOLVED-IN-TREE` — All 5 dead imports are confirmed deleted in the current
  working tree, and the deletion is gate-proven. Fresh evidence this session:
  - `grep -n 'AsBytes\|VariantIterator\|IntoEnumIterator' src/{geometry,buffer,ubo}.rs` → exactly 2
    hits, both the *legitimate* fully-qualified uses (`buffer.rs:43` and `ubo.rs:16`, `Data :
    mem::AsBytes + ?Sized`); zero bare-import hits. `git diff` on the 3 files shows the uncommitted
    deletions: `geometry.rs` line 4 dropped `AsBytes` (and nothing else); `buffer.rs` line 4
    dropped `AsBytes` + `StrideTrait`; `ubo.rs` line 3 dropped `AsBytes`, `IntoEnumIterator`,
    `VariantIterator`.
  - **Attribution caveat:** this workspace hosts concurrent uncommitted work from the broader
    058-sweep effort; the deletions landed as part of the concurrent minwebgl sweep tranche (16
    files, +191/−101 in the crate's diff), not via a standalone execution of this task file. With
    zero commit history on these lines, per-actor attribution is not possible — recorded factually.
  - **Verification (this session, independent of whoever edited):** `cargo clippy -p minwebgl
    --no-deps --all-targets --all-features -- -D warnings` → **exit 0**, 51s
    (`module/min/minwebgl/-0001_longrun.log`). Under `-D warnings` any surviving `unused_imports`
    site would fail the gate — green is positive proof all 5 are gone and no new ones appeared.
  - Task 062's AF1 re-check contract satisfied by the "resolved by deletion" ending: `grep -n
    AsBytes module/min/minwebgl/src/geometry.rs` → 0 hits. Awaits independent
    verification/promotion per the task lifecycle.
- **[2026-08-11]** `FILED` — Filed via lightweight Draft capture
  (`tsk.rulebook.md § Core Procedures : Procedure - Draft Task`, PROC8) during this session's TA106
  out-of-scope-findings triage. Site 1 classified via `tsk.rulebook.md § Task File : Deduplication
  Search` as Case E (closed task 062 already names this exact site, but its own scope explicitly
  excludes fixing it). Sites 2-3 are a fresh discovery from this session's own direct `cargo check`
  output, confirmed via `grep -rl "buffer.rs.*AsBytes\|ubo.rs.*AsBytes\|VariantIterator" task/` to
  have no prior mention anywhere in `task/` outside raw, non-authoritative compiler-output log
  files (`task/unverified/-00NN_longrun.log`). Folded into this one task rather than filed
  separately: same crate, same defect class (dead imports), same trivial fix shape.
| 2026-08-11 18:25:50 | user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/ | CLAIM_VERIFY | verification claimed |
- **[2026-08-11]** `ACCEPTED` — Independent acceptance-verification pass (self, Tier 2 Dual-Role
  Self-Check — distinct from and subsequent to the readiness-gate self-check recorded in
  `## Verification Record` below) walked every Checklist/Measurement/Invariant/Anti-faking item
  against direct, fresh inspection of the live files, a fresh clippy re-run, and the fix commit.
  Verdict: **ACCEPT**.
  **C1-C3:** confirmed via direct read — `geometry.rs`'s `use` statement (line 4) carries no
  `AsBytes`; `buffer.rs`'s `use` statement (line 4) carries no bare `AsBytes` (qualified
  `mem::AsBytes` at line 43 untouched); `ubo.rs`'s `use` statement (line 3) carries none of
  `AsBytes`/`VariantIterator`/`IntoEnumIterator` (qualified `mem::AsBytes` at line 16 untouched).
  **C4:** both legitimate qualified uses present.
  **C5/I1/I2:** fresh `cargo clippy -p minwebgl --no-deps --all-targets --all-features -- -D
  warnings` → **exit 0** (`-0296_longrun.log`, 53s combined run alongside the previously-excluded
  058-sweep cone recheck); `git show --stat 96bb2aef -- module/min/minwebgl/src/geometry.rs
  module/min/minwebgl/src/buffer.rs module/min/minwebgl/src/ubo.rs` → exactly 3 files, 4
  insertions/6 deletions, no other file. Same batched-commit path-scoping interpretive note as task
  082's own I1/I2 (`96bb2aef` is a large multi-purpose consolidation commit; "touches only these 3
  files" is evaluated path-scoped to `module/min/minwebgl/src/`, not the commit's full unscoped
  diff).
  **M1:** fresh `grep -n 'AsBytes\|VariantIterator\|IntoEnumIterator' geometry.rs buffer.rs ubo.rs`
  → exactly 2 hits, both the legitimate fully-qualified `mem::AsBytes` uses (`buffer.rs:43`,
  `ubo.rs:16`) — matches the task's own MET bar exactly.
  **AF1:** `grep -rn 'allow(unused_imports)'` near the 3 sites → no hits, confirms the fix is
  genuine deletion, not lint suppression.
  **AF2:** task 062's own AF1 re-check contract ("`grep -n AsBytes geometry.rs` must show more than
  1 hit before resolved") is satisfied by the stricter actual outcome — 0 hits, resolved via
  deletion rather than by adding a new use, a valid (stricter) resolution of the same underlying
  drift.
  No blocking findings.

## Verification Record

**Gate Round 1** (Tier 2 — Dual-Role Self-Check, one-shot, self-administered by
user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/)

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Scope Coherence | — | 🟢 | — | — |
| D2 | MOST Goal Quality | — | 🟢 | — | — |
| D3 | Value / YAGNI | — | 🟢 | Deliverable already committed (`96bb2aef`) before this task's own filing; remaining value is formal verification/closure of the fix and of task 062's own still-open AF1 drift finding, not new execution — legitimate, not YAGNI (same pattern as 082/085/087/090). | — |
| D4 | Implementation Readiness | — | 🟢 | — | — |
| D5 | Execution Scope | — | 🟢 | — | — |
| D6 | Crate Scope Unity | — | 🟢 | — | — |
| D7 | Crate Locality | — | 🟢 | — | — |
| D8 | Crate Single Responsibility | — | 🟢 | — | — |
| **Total** | | — | 🟢 | 1 non-blocking | — |

Confirming pass: D1 — In/Out of Scope both non-empty, meaningful observable outcome (5
compiler-provable dead-import tokens across 3 files), Scope Sizing Gate passes. D2 — Motivated (real,
if minor, lint-debt/hygiene concern already flagged once before by task 062's own AF1), Observable
(exact file+line+token named), Scoped (one crate), Testable (exact clippy command + grep stated). D3
— Null Hypothesis: skipping leaves task 062's own AF1 finding formally unresolved in the tracker even
though the code is already fixed; concrete, not speculative. D4 — Acceptance Criteria concrete and
traceable; Test Matrix correctly omitted with justification (no behavior to assert). D5/D6 — all
paths resolve inside this repo, inside the single `minwebgl` crate. D7 — `minwebgl` is a concrete
leaf WebGL-binding crate, not an aggregator. D8 — deletion-only change, no new concern grafted onto
the crate.

Adversarial pass: D1 — checked whether this is too small to be a meaningful unit (anti-tiny-task);
rejected — compiler-gate-provable, cross-referenced to a specific pre-existing finding (062's AF1),
consistent with other similarly-scoped completed tasks this session. D2 — checked whether "Motivated"
is undercut by the fact the crate's clippy gate is *currently* green (fix already landed); the
Goal's present-tense framing describes the originally-discovered problem, not live state — same
established convention as task 082's Goal wording, not a defect. D3 — sharpest attack: since
`cargo clippy -p minwebgl --no-deps --all-targets --all-features -- -D warnings` was independently
re-run fresh this round (exit 0, 3.46s) and `git status --short` on the 3 files is empty, is there
*any* remaining work this task performs? No new execution — but formal verification/closure of an
already-landed fix, closing a specific named pre-existing tracker gap (062's AF1), is real,
non-speculative value under this lifecycle; flagged as Non-Blocking for the Acceptance Verifier
rather than silently dropped. D4 — independently re-derived, not trusted from prose: direct `sed`
read of `geometry.rs:4`/`buffer.rs:4`/`ubo.rs:3`'s current `use` lines confirms zero occurrence of
`AsBytes`/`VariantIterator`/`IntoEnumIterator` among the imports; direct grep confirms exactly 2
remaining hits, both the legitimate fully-qualified `mem::AsBytes` uses (`buffer.rs:43`,
`ubo.rs:16`); `git log -1` on the 3 files resolves to commit `96bb2aef`, matching task 082's
independently-confirmed fix commit. No discrepancy found. D5/D6/D7/D8 — re-scanned for any
foreign-repo/foreign-crate/aggregator-crate concern; none found, same conclusion as the confirming
pass.
