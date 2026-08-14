# Fix mdmath_core IterMut aliasing/UB soundness bug

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
- **unit:** lib/yrd_gamedev/cgtools/module/math/mdmath_core
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/
- **verification_date:** 2026-08-10
- **blocked_by:** null

## Goal

Fix a soundness bug in `mdmath_core`'s mutable-iterator code identified during the workspace-wide audit
(P1 — soundness bucket, Fix-in-place): an `IterMut`-style construct produces aliased mutable
references/undefined behavior under certain access patterns. **Carried forward from the audit triage
plan — exact file/line was cited in the delivered plan but is not re-verified in this filing pass;
re-confirm the precise citation against current `module/math/mdmath_core/src/` before making any change**,
per the plan's own ground rule that findings must be re-confirmed immediately before the file they cite is
touched. Write a failing test demonstrating the aliasing (e.g. via `miri` or a targeted borrow-check-evading
pattern) before fixing.

## In Scope

- `module/math/mdmath_core/src/vector/tuple2.rs`: `Tuple2IterMut`'s `next()`/`next_back()` — replace
  the shared `index` counter with independent `front`/`back` cursors
- `module/math/mdmath_core/src/vector/tuple3.rs`: `Tuple3IterMut`, same fix
- `module/math/mdmath_core/src/vector/tuple4.rs`: `Tuple4IterMut`, same fix
- `module/math/mdmath_core/tests/inc/vector_test/tuple2_test.rs`, `tuple3_test.rs`, `tuple4_test.rs`:
  one test per file demonstrating the aliasing via an interleaved `.next()`/`.next_back()` sequence,
  failing (wrong values, and genuine UB under Miri) before the fix, passing after

## Out of Scope

- `Tuple1IterMut` (`module/math/mdmath_core/src/vector/tuple1.rs`) — investigated and hand-traced;
  NOT exploitable. With exactly 1 element, `front`/`back` converge after exactly one call regardless
  of call direction, so no aliasing is structurally possible. Left unchanged.
- `Tuple0` (0-element tuple) — trivial, no elements to alias
- Array (`[E;N]`) and slice (`[E]`) `VectorIterMut` impls (`array.rs`, `slice.rs`) — confirmed they
  delegate to `core::slice::IterMut`, already sound; not touched
- Any change to the public `VectorIterMut`/`VectorIterator` trait surface — `TupleNIterMut` structs
  are private implementation details, not exposed via `mod_interface!`

## Requirements

-   All work must strictly adhere to all applicable rulebooks (discover via `kbase .rulebooks`)

## Delivery Requirements

Unordered constraints. Execution order determined by the governing plan (if any),
not by this section.

-   Test Matrix populated before any test code
-   Every Test Matrix case is backed by a test that failed (wrong values under `cargo nextest`, and
    confirmed genuine UB under `cargo +nightly miri test`) before its implementing change landed
-   Minimum code to satisfy Test Matrix — no new public API, no behavior change to correct
    single-direction or `.rev()`-only iteration
-   `cargo nextest run -p mdmath_core --all-features` passes with zero failures (package-scoped,
    never full workspace)
-   `cargo +nightly miri test` (package-scoped) shows zero Stacked-Borrows UB on the affected tests
-   `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings` clean
-   No duplication introduced; `// SAFETY:` comments (required by the crate's
    `undocumented_unsafe_blocks = "deny"` lint) updated to justify the new front/back invariant
-   All Rust code uses 2-space indentation, no `cargo fmt`

## Work Procedure

1. Grep `module/math/mdmath_core/src/` for `IterMut`, `unsafe`, raw-pointer casts (`as *mut`,
   `as *const`), `from_raw_parts_mut`, `split_at_mut` to enumerate every hand-rolled mutable-iterator
   construct.
2. For each `TupleNIterMut` found, hand-trace `next()`/`next_back()` under a mixed-direction call
   sequence to determine whether the shared `index` counter can double-yield a field as two live
   `&mut` references.
3. Write one failing test per exploitable arity (interleaved `.next()`/`.next_back()` calls, assert
   final values match a correct front/back traversal); confirm it fails under
   `cargo nextest run -p mdmath_core --all-features` with the predicted aliased values, and confirm
   genuine UB under `cargo +nightly miri test` (Stacked Borrows).
4. Fix each exploitable `TupleNIterMut` by replacing the shared `index : usize` field with
   independent `front`/`back : usize` cursors, guarded by `if front >= back { return None }`,
   mirroring `core::slice::IterMut`'s design; update `// SAFETY:` comments accordingly.
5. Re-run `cargo nextest run -p mdmath_core --all-features`, `cargo +nightly miri test` (filtered to
   the new tests), and `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings`;
   confirm all pass/clean.

## Test Matrix

| # | Input Scenario | Config Under Test | Expected Behavior |
|---|---|---|---|
| T01 | `.next()` then `.next_back()` on `(i32,i32)` | `Tuple2IterMut` | Yields disjoint fields (0 then 1); no aliasing; Miri clean |
| T02 | `.next()`, `.next()`, `.next_back()` on `(i32,i32,i32)` | `Tuple3IterMut` | Yields disjoint fields (0,1,2); no aliasing; Miri clean |
| T03 | `.next()`, `.next_back()`, `.next()`, `.next_back()` on `(i32,i32,i32,i32)` | `Tuple4IterMut` | Yields disjoint fields (order 0,3,1,2); no aliasing; Miri clean |
| T04 | `Tuple1IterMut` under any call sequence | 1-element tuple | Hand-trace confirms: front/back always converge after 1 call — not exploitable, left unchanged |
| T05 | Pure `.next()`-only or pure `.rev()`-only iteration (pre-existing tests) | All tuple arities | Unchanged behavior — regression check |

## Acceptance Criteria

- `Tuple2IterMut`/`Tuple3IterMut`/`Tuple4IterMut` use independent `front`/`back` cursors; front and
  back provably never cross
- Interleaved `.next()`/`.next_back()` sequences yield disjoint elements for all three arities,
  confirmed by passing tests
- `cargo +nightly miri test` shows zero Stacked-Borrows UB on the affected tests
- Existing pure-forward and pure-`.rev()` behavior is unchanged for every tuple arity
- `cargo nextest run -p mdmath_core --all-features` passes with zero failures
- `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings` clean

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
| B2 | Test-First Requirement | — | 🟢 | — | — |
| B3 | Evidence of Failure | — | 🟢 | Pre-fix aliased values and genuine Miri UB both directly reproduced (see History) | — |
| B4 | Proper Fix Only | — | 🟢 | — | — |
| B5 | Fix Verification | — | 🟢 | Directly re-ran `cargo +nightly miri test -p mdmath_core --all-features` (76 passed, 0 failed — zero UB crate-wide, not just the 3 new tests), `cargo nextest run -p mdmath_core --all-features` (76/76 passed), `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings` (clean), all via `longrun`, 2026-08-10 | — |
| B6 | Knowledge Preservation | — | 🟢 | Confirmed by direct read: `tuple2.rs:161-169` carries the mandated 3-field `Fix(BUG-050)`/`Root cause`/`Pitfall` comment; `tuple2_test.rs`'s `bug_reproducer(BUG-050)` test carries the mandated 5-section doc comment (`Root Cause`/`Why Not Caught`/`Fix Applied`/`Prevention`/`Pitfall`) | — |
| B7 | Code Cleanliness | — | 🟢 | — | — |
| **Total** | | 🔴 | 🟢 | 0 | 0/0 |

**Aggregate verdict:** PASS — all 15 dimensions clean on both the confirming and adversarial pass, no Blocking Findings. D1–D8 use `tsk` skill's Readiness dimensions; B1–B7 use the Bug-Fixing Task Quality Requirements (this task fixes a P1 soundness bug, so both apply). Verification independently re-executed (Miri + native nextest + clippy, all package-scoped, all this session) rather than solely trusted from the implementing pass's own prose, per this session's Stale Evidence Trust discipline. **Byproduct, out of this task's own scope:** the same full-crate Miri sweep that reconfirmed this fix also surfaced an entirely unrelated, pre-existing UB defect in `vector/slice.rs`'s `vector_mut` (wrong pointer accessor, not a shared-cursor aliasing pattern) — filed, fixed, and closed separately as `BUG-054`; noted here only for traceability, not part of this task's own deliverable.

## Verification

### Checklist

- [x] C1 — Do `Tuple2IterMut`/`Tuple3IterMut`/`Tuple4IterMut` use independent `front`/`back` cursors instead of the original shared `index`? `grep -n "front : usize\|back : usize\|index : usize" module/math/mdmath_core/src/vector/tuple{2,3,4}.rs` → each `TupleNIterMut` struct declares `front : usize` and `back : usize`; the only remaining `index : usize` fields belong to the separate, immutable `TupleNIter` types (aliasing a shared `&E` is harmless), never to a `*IterMut` type.
- [x] C2 — Does each fixed file carry the mandated `Fix(BUG-050)` 3-field comment directly above its `TupleNIterMut` struct? Direct read: `tuple2.rs:156-164` (struct at 165), `tuple3.rs:158-166` (struct at 167), `tuple4.rs:168-176` (struct at 177) — all three carry `Fix(BUG-050)` / `Root cause` / `Pitfall` fields.
- [x] C3 — Is `Tuple1IterMut` still present and "left unchanged" as this task's own Out of Scope section claims? NO LONGER — direct read of current `tuple1.rs` shows `Tuple1IterMut` was deleted entirely (by a separate, later task, `059`, per its own History) and replaced with `core::iter::once`. This does not contradict 009's own work: 009's Scope never lists `tuple1.rs` as touched, and 059's own History independently confirms it, not 009, made that change. Recorded here so "left unchanged" isn't misread as still true of the crate today.
- [x] C4 — Do the array/slice `VectorIterMut` impls remain untouched, still delegating to `core::slice::IterMut` as claimed Out of Scope? `array.rs:52-59` and `slice.rs:78-87` both call `<[E]>::iter_mut(self)` directly — no raw-pointer code, confirms neither was touched by this fix.
- [x] C5 — Do the three T01-T03 reproducer tests still exist? `grep -n "bug_reproducer(BUG-050)"` → present at `tuple2_test.rs:94`, `tuple3_test.rs:105`, `tuple4_test.rs:116`.
- [x] C6 — Are `Tuple2IterMut`/`Tuple3IterMut`/`Tuple4IterMut` still private, with no new public API (per Acceptance Criteria)? `grep -rn "Tuple2IterMut\|Tuple3IterMut\|Tuple4IterMut" module/math/mdmath_core/src/` → hits only inside each type's own defining file; zero references from any `mod_interface!` block or other file.

### Measurements

- [x] M1 — Shared `index : usize` cursor fields remaining across the three `*IterMut` types: `0` (was: `3`, one per type — `git show 9b71cf39^:module/math/mdmath_core/src/vector/tuple2.rs` lines 161-165 shows `struct Tuple2IterMut { tuple: ..., index: usize }`; `tuple3.rs`/`tuple4.rs` at the same parent commit carry the identical shape).
- [x] M2 — Crate test count: `89` passed, this session's own fresh run (was: `76` immediately pre-fix per this task's own History; fix-neutral — this task's 3 new reproducers didn't move the 76 baseline, the later climb to 89 is task 059's unrelated additions, independently reconfirmed current in I1).

### Invariants

- [x] I1 — Test suite (crate-scoped): `cargo nextest run -p mdmath_core --all-features` (via `longrun`) → exit 0, 89 tests run: 89 passed, 0 skipped, including all 3 BUG-050 reproducers (log `-0014_longrun.log`).
- [x] I2 — Lints clean: `cargo clippy -p mdmath_core --all-targets --all-features -- -D warnings` (via `longrun`) → exit 0, zero warnings (log `-0018_longrun.log`).
- [x] I3 — Miri Stacked Borrows (this task's own original acceptance criterion): `cargo +nightly miri test -p mdmath_core --all-features` (via `longrun`) → exit 0, 89 passed, 0 failed, zero UB detected crate-wide, including all 3 mixed-direction reproducers (log `-0019_longrun.log`); Miri's availability was confirmed first (`cargo +nightly miri --version` → `miri 0.1.0`), so this is a genuine re-run, not a substitute.

### Anti-faking checks

- [x] AF1 — Guards against the shared-`index` aliasing pattern silently returning (e.g. a future refactor collapsing `front`/`back` back into one field): re-running C1's grep must keep showing `front`/`back` and zero `index` on all three `*IterMut` structs, and the 3 `bug_reproducer(BUG-050)` tests must keep passing under both native `cargo nextest` and `cargo +nightly miri test`.
- [x] AF2 — Guards against `Tuple1IterMut`'s later deletion (task 059's work) being mistaken for part of *this* task's own deliverable, or vice versa: `git log --oneline -- module/math/mdmath_core/src/vector/tuple1.rs` cross-referenced against `task/completed/059_mdmath_core_marker_resolution.md`'s own History is the re-check if this attribution is ever disputed.

## History

- **[2026-08-08]** `FILED` — Filed from workspace-wide Delete/Rewrite/Fix triage plan, P1 (soundness bugs)
  tier, Fix-in-place bucket.
- **[2026-08-10]** — Re-confirmed citation via grep + hand-trace: `Tuple2IterMut`/`Tuple3IterMut`/
  `Tuple4IterMut` (`module/math/mdmath_core/src/vector/tuple{2,3,4}.rs`) shared one `index : usize`
  field between `next()`/`next_back()`, each with per-arity hardcoded match arms — interleaving the
  two directions on the same (non-`.rev()`) iterator double-yielded an already-returned tuple field
  as a second, simultaneously-live `&mut E` reference. `Tuple1IterMut` (`tuple1.rs`) hand-traced and
  confirmed NOT exploitable (front/back always converge after exactly one call for a single element,
  in either direction); array/slice paths confirmed to delegate to std's sound `iter_mut()`. TDD:
  added one interleaved-call test per exploitable arity; pre-fix
  `cargo nextest run -p mdmath_core --all-features` reproduced the exact predicted aliased values
  (tuple2 actual `(200,43)` vs expected `(100,200)`; tuple3 actual `(300,200,44)` vs expected
  `(100,200,300)`; tuple4 actual `(400,43,300,45)` vs expected `(100,300,400,200)`); pre-fix
  `cargo +nightly miri test -- disjoint` confirmed genuine Stacked Borrows UB on the tuple2 case
  ("attempting a write access ... tag does not exist in the borrow stack for this location"). Fix:
  replaced the shared `index` counter with independent `front`/`back : usize` cursors (mirrors
  `core::slice::IterMut`), guarded by `front >= back`, in all three files; updated `// SAFETY:`
  comments to justify the new invariant. Post-fix: `cargo nextest run -p mdmath_core --all-features`
  → 76 tests run, 76 passed, 0 skipped; `cargo +nightly miri test -- disjoint mixed_direction` → 3
  passed, 0 failed (zero UB on any of the three arities); `cargo clippy -p mdmath_core --all-targets
  --all-features -- -D warnings` → clean. Note: during this same session, another process
  concurrently touched `tuple{2,3,4}.rs` and their test files to add formal `Fix(BUG-050)`-tagged
  source/test documentation on top of the same front/back fix and test scenarios (no logic conflict;
  one redundant reproducer test in `tuple2_test.rs` was consolidated to a single test by that
  process) — final state re-verified and matches the numbers above.
- **[2026-08-10]** `GATE_CHECKED_AND_COMPLETED` — Self-administered Tier 2 Dual-Role Self-Check (see
  `## Verification Record`): directly re-read `tuple2.rs`'s front/back cursor fix and its
  `Fix(BUG-050)` comment, and `tuple2_test.rs`'s `bug_reproducer(BUG-050)` 5-section doc comment,
  rather than relying solely on the prior entry's own prose. Independently re-ran
  `cargo +nightly miri test -p mdmath_core --all-features` (76 passed, 0 failed — crate-wide, zero
  UB), `cargo nextest run -p mdmath_core --all-features` (76/76 passed), and `cargo clippy -p
  mdmath_core --all-targets --all-features -- -D warnings` (clean), all via `longrun`. All 15
  dimensions (8 Readiness + 7 Bug-Fixing Quality) PASS on both passes, zero Blocking Findings. Note:
  the header's `state`/`verified_by`/`verification_date` fields had already been set to ✅/self/
  2026-08-10 by an earlier pass in this same session, but no `## Verification Record` existed yet and
  this entry's own closing line still read "State left at 📝 Draft" — an internal inconsistency
  caught and reconciled by this gate-check pass, not a new decision to complete the task. State
  confirmed ✅ Completed; file relocated `draft/` → `completed/` to match.
