# BUG-510: `check_dependency_cycle` in `shader_chunks_validate_core` reports only the first dependency cycle, silently dropping every other independent one

- **Severity:** Medium (a validation/linting tool silently under-reporting a structural registry
  defect it explicitly promises to catch -- no data corruption or crash, but a second, unrelated
  cycle in the bundled chunk registry would pass `shader_chunks validate` completely undetected
  until the first-reported cycle is fixed and the tool is re-run)
- **state:** Completed
- **Affects:** `shader_chunks_validate_core::validate`/`validate_registry` (the
  `check_dependency_cycle` internal check), and transitively `shader_chunks_validate::validate`/
  `validate_chunks` (the `shader_chunks validate` CLI command built on it)
- **Component:** `module/shader/shader_chunks_validate_core` (`src/lib.rs`)
- **repo_identity:** self
- **Filed:** 2026-08-21
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-21
- **Fixed:** 2026-08-21
- **Related Bugs:** None -- found during a first, dedicated sweep of
  `shader_chunks_validate_core`/`shader_chunks_validate`/`shader_chunks_render_core` (previously
  unswept siblings of the already-swept `shader_chunks_query`/`shader_chunks_query_core`/
  `shader_chunks_core`/`shader_chunks_params`/`shader_chunks_params_core`).

## Symptom

```rust
// pre-fix -- shader_chunks_validate_core/src/lib.rs
fn check_dependency_cycle( chunks : &[ ChunkDescriptor ] ) -> Vec< Finding >
{
  match shader_chunks_core::set_try_compose( chunks )
  {
    Err( shader_chunks_core::ComposeError::CyclicDependency( trail ) ) => vec!
    [
      Finding { chunk : "(registry)".to_string(), check : "dependency_cycle", message : format!( "cyclic dependency: {trail}" ) },
    ],
    Ok( _ ) | Err( shader_chunks_core::ComposeError::MissingDependency { .. } ) => vec![],
  }
}
```

A single call to `shader_chunks_core::set_try_compose` over the *whole* chunk set was treated as
if it surveyed every chunk for cycles. It does not: `set_try_compose`'s own
`entries_sort_and_join` helper (`shader_chunks_core/src/lib.rs`) runs
`for entry in entries { visit( ... )?; }` and returns on the *first* `Err` its depth-first `visit`
walk produces -- so as soon as one cycle is found, the whole call returns, and every chunk not yet
visited (including a second, completely independent cycle elsewhere in the set) is never looked
at.

## Impact

**Who is affected:** Anyone running `shader_chunks validate` (or calling
`shader_chunks_validate_core::validate`/`validate_registry` directly) against a chunk set that
happens to contain two or more structurally independent `depends_on` cycles at once.

**What breaks:** the tool's own documented, load-bearing contract: `shader_chunks_validate_core/readme.md`
promises "Five independent, non-panicking checks run across every bundled chunk in one pass and
report every problem found, rather than failing loudly (`compose`'s panic) or stopping at the
first one"; `shader_chunks_validate/docs/cli/command_group/01_validate.md`'s own stated
**Invariant** is "Every check runs over the full input set in one pass; no check short-circuits or
gets skipped because another check already found a related problem in the same chunk."
`check_dependency_cycle` violated both: it silently stopped after the first cycle, exactly the
behavior both documents explicitly rule out. A shader author fixing the first reported cycle and
re-running `validate` would see a clean pass even though a second, entirely unrelated cycle was
still present -- the class of "fixed one thing, tool says all-clear, but it wasn't" failure this
whole command exists to prevent.

**Magnitude:** every independent dependency cycle beyond the first, in the same
`validate`/`validate_registry` call, went completely unreported (not merely under-detailed --
genuinely never visited).

**Entity Scope:** None -- a code-level defect in the linting engine.

## How Discovered

Dedicated first-sweep of `shader_chunks_validate_core` (previously unswept). While reading
`check_dependency_cycle` next to its sibling checks' own doc comments -- `check_missing_dependencies`'s
doc comment explicitly states it reports "every instance across the whole set, not just the first
one `dependency_closed`'s boolean would find" -- noticed `check_dependency_cycle` makes no
equivalent claim and instead delegates entirely to a single `shader_chunks_core::set_try_compose`
call. Traced `set_try_compose` -> `entries_sort_and_join` -> `visit` in
`shader_chunks_core/src/lib.rs` and confirmed the `for entry in entries { visit( ... )?; }` loop
returns via `?` on the first `Err`, so the walk never reaches chunks beyond the first-found cycle.
Cross-checked against the crate's own readme.md and the CLI command_group doc's stated
Invariants, both of which promise otherwise.

## Minimum Reproducible Example

```rust
// shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs
let findings = validate( &[ LOCAL_CYCLE_A, LOCAL_CYCLE_B, LOCAL_CYCLE_C, LOCAL_CYCLE_D ] );
// LOCAL_CYCLE_A <-> LOCAL_CYCLE_B is one cycle; LOCAL_CYCLE_C <-> LOCAL_CYCLE_D is a second,
// structurally independent cycle -- no chunk shared between the two pairs.
let cycles : Vec< _ > = findings.iter().filter( | f | f.check == "dependency_cycle" ).collect();
assert_eq!( cycles.len(), 2 ); // pre-fix: 1 -- the C<->D cycle was never visited at all
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/shader/shader_chunks_validate_core && cargo nextest run -p shader_chunks_validate_core dependency_cycle_reports_every_independent_cycle_not_just_the_first
```

## Root Cause

`check_dependency_cycle` reused `shader_chunks_core::set_try_compose`'s single topological-sort
pass wholesale, as a single `match` on one call. `set_try_compose`'s underlying `visit` walk
short-circuits on the first cycle it finds (a correct, deliberate design for `compose`'s own
panic-on-first-problem contract, per `shader_chunks_core/src/lib.rs`'s own doc comments on
`compose`/`try_compose`), but `check_dependency_cycle` inherited that short-circuiting behavior
into a check whose own crate promises the opposite: report every problem, don't stop at the first
one.

## Why Not Caught

The only existing dependency-cycle fixture (`LOCAL_CYCLE_A`/`LOCAL_CYCLE_B`) exercised exactly one
cycle in isolation (`dependency_cycle_is_reported_and_not_duplicated_as_wgsl_compile_failure`),
and the real bundled `shader_chunks_core::CHUNKS` registry has zero cycles. No fixture ever
combined two independent cycles in the same `validate` call, so `set_try_compose`'s "stop at the
first `Err`" behavior was never exercised through `check_dependency_cycle`'s own "report every
problem" contract.

## Fix Applied (2026-08-21)

**`module/shader/shader_chunks_validate_core/src/lib.rs`:**
`check_dependency_cycle` now loops instead of matching once: each `Err( CyclicDependency( trail ) )`
is recorded as a `Finding`, then the specific chunk that closed that cycle -- the trailing name in
`ComposeError::CyclicDependency`'s own documented `"[...] -> name"` trail format -- is removed
from a local working copy of the chunk set, and `set_try_compose` is re-run on the shrunken set to
look for further, independent cycles. Removing a cycle's culprit chunk can leave an otherwise-
innocent dependent pointing at a now-missing name; that surfaces as a collateral
`ComposeError::MissingDependency` here (not a real registry problem -- `check_missing_dependencies`
already reports every genuine instance against the full, untouched input), so it is silently
absorbed the same way, by removing the affected dependent and continuing. Each iteration removes
exactly one chunk, so the loop is bounded by `chunks.len()` and always terminates. Added a
`Fix(BUG-510)`/`Root cause`/`Pitfall` source comment directly above the function.

**New regression test** (`tests/shader_chunks_validate_core_test.rs`):
`dependency_cycle_reports_every_independent_cycle_not_just_the_first` -- two new fixtures
(`LOCAL_CYCLE_C`/`LOCAL_CYCLE_D`, a second cycle structurally independent of the existing
`LOCAL_CYCLE_A`/`LOCAL_CYCLE_B`) combined in one `validate` call, asserting exactly 2
`dependency_cycle` findings are returned, one naming each pair.

## Verification

`longrun`-detached, from repo root.

- **Pre-fix (RED):** `cargo nextest run -p shader_chunks_validate_core dependency_cycle_reports_every_independent_cycle_not_just_the_first`
  against the pristine source -- `0 passed, 1 failed`, `assert_eq!` reporting `left: 1, right: 2`
  (only the A/B cycle was found; C/D's independent cycle was silently never visited), confirming
  the defect exactly as diagnosed.
- **Post-fix (GREEN):** `cargo nextest run -p shader_chunks_validate_core` (full crate suite) --
  `9 tests run: 9 passed, 0 skipped`, including both the new test and the pre-existing single-cycle
  test (`dependency_cycle_is_reported_and_not_duplicated_as_wgsl_compile_failure`, confirming no
  regression to the single-cycle/no-derivative-`wgsl_compile`-noise case) and
  `validate_registry_reports_nothing_for_the_current_bundled_registry` (confirming the real bundled
  registry, which has zero cycles, is unaffected).
- **Dependent crate:** `cargo nextest run -p shader_chunks_validate -p shader_chunks_validate_core`
  -- `13 tests run: 13 passed, 0 skipped` (confirms the CLI wrapper crate, which depends on this
  engine, is unaffected).
- **Clippy:** `cargo clippy -p shader_chunks_validate_core --all-targets --all-features -- -D
  warnings` -- clean.

## Generalized Version

Any check in this crate (or a similar one) that reuses a single-shot engine function from a
lower-level crate must be verified against a fixture combining *two* independent instances of the
problem it claims to catch, not just one -- a single-instance fixture cannot distinguish "reports
every instance" from "reports the first instance and gets lucky because there was only one." This
matches the bar the other four checks in this same file already meet (`check_manifest_drift`,
`check_duplicate_names`, `check_missing_dependencies` all iterate the full input unconditionally;
only `check_dependency_cycle` delegated to a function with early-return semantics without
compensating for it).

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-21 | filed + fixed + verified | Found during a dedicated first sweep of `shader_chunks_validate_core`/`shader_chunks_validate`/`shader_chunks_render_core` (siblings of the already-swept `shader_chunks_query`/`shader_chunks_query_core`/`shader_chunks_core`/`shader_chunks_params`/`shader_chunks_params_core`). Root cause: `check_dependency_cycle` delegated entirely to one `shader_chunks_core::set_try_compose` call, whose underlying `visit` walk returns on the first cycle found, so any second, independent cycle in the same input was never visited -- silently contradicting this crate's own "report every problem, don't stop at the first one" contract (stated in its readme.md and in `shader_chunks_validate/docs/cli/command_group/01_validate.md`'s Invariants). Fixed by looping `set_try_compose` calls, removing each detected cycle's culprit chunk (parsed from the error's own documented trail format) and re-checking the shrunken set until clean, absorbing any collateral `MissingDependency` this produces along the way. Verified RED (`1` finding, not `2`) against the pristine source, then GREEN post-fix across the full `shader_chunks_validate_core` suite (9/9) and the dependent `shader_chunks_validate` crate (13/13 combined), plus clean clippy. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Confirming pass: ran the new test against the pristine (pre-fix) source and observed a genuine failure (`left: 1, right: 2`), then against the fixed source and observed a genuine pass -- not a tautological assertion. Adversarial pass: manually traced the fix's loop through both fixture pairs by hand (iteration-by-iteration: cycle A/B detected and removed, collateral `MissingDependency` for B's now-dangling reference absorbed, cycle C/D detected and removed, collateral `MissingDependency` for D absorbed, terminates `Ok` on the empty set) and confirmed the trace matches the actual `PASS` output -- no gap between claimed and actual behavior. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-510)`/`Root cause`/`Pitfall` 3-field source comment directly above `check_dependency_cycle`; 5-section doc comment (Root Cause/Why Not Caught/Fix Applied/Prevention/Pitfall) on the regression test itself. | — |
| D3 | Scope containment | — | 🟢 | Fix confined to `check_dependency_cycle` in `shader_chunks_validate_core/src/lib.rs` and its own test file; no changes to `shader_chunks_core` (the reused engine, out of this sweep's target area) or to public API signatures -- `Finding`, `validate`, `validate_registry` are all unchanged, so no downstream crate needed updating beyond re-running its own tests to confirm. | — |

**Reproduced:** YES -- `cargo nextest run -p shader_chunks_validate_core
dependency_cycle_reports_every_independent_cycle_not_just_the_first` failed with `left: 1, right:
2` against the pristine source (2026-08-21), confirming only one of the two independent cycles was
ever reported; the same command passes post-fix.

## Refs: src/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_validate_core/src/lib.rs` | `check_dependency_cycle` now loops over `set_try_compose`, removing each detected cycle's culprit chunk and re-checking, instead of matching a single call once. Added `Fix(BUG-510)`/`Root cause`/`Pitfall` source comment. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs` | Added `LOCAL_CYCLE_C`/`LOCAL_CYCLE_D` fixtures (a second, independent cycle) and `dependency_cycle_reports_every_independent_cycle_not_just_the_first`, tagged `bug_reproducer(BUG-510)`. |
