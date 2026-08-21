# BUG-421: `shader_chunks_params_core`'s `range_clause_parse` accepts an inverted `range(min, max)` clause with `min > max`

- **Severity:** Low (authored-content-only input -- chunk manifests are trusted, not adversarial --
  but produces a `Parameter` carrying a mathematically nonsensical range with no error, silently
  propagating into any downstream consumer of `discover`/`chunk_discover`, e.g. a slider UI)
- **state:** Completed
- **Affects:** `shader_chunks_params_core::discover`/`chunk_discover` and any consumer of their
  `Parameter::range` output (e.g. `shader_chunks_params`'s `tunables` table, or a future slider/UI
  binding) for a chunk whose `//@ param:` line declares an inverted `range(min, max)` clause
- **Component:** `module/shader/shader_chunks_params_core` (`src/lib.rs`, `range_clause_parse`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-20
- **Related Bugs:** None -- distinct mechanism and distinct field (cross-field relational
  validation on `range(min, max)`) from BUG-293 (`param_lines`'s `//@ param:` prefix-recognition
  leniency, same crate, same file, different function).

## Symptom

```rust
// pre-fix
let params = discover( "//@ param: x argument u32 range(8, 1)\n" );
// params[0].range == Some(( Range { min: 8.0, max: 1.0 }, RangeSource::Declared )) -- no panic
```

## Impact

**Who is affected:** Any consumer of `discover`/`chunk_discover`'s `Parameter::range` field for a
chunk whose `//@ param:` line has an inverted `range(min, max)` clause -- currently
`shader_chunks_params`'s `tunables` table (renders the inverted range verbatim, e.g. `8..1`, with no
indication it's malformed), and any future consumer that treats `range.min`/`range.max` as bounds for
a slider or clamp (e.g. `value.clamp( range.min, range.max )` panics in Rust for an inverted range,
or silently misbehaves in a UI slider).

**What breaks:** No panic at parse time (every other malformed shape -- missing `range(...)`, missing
comma, non-numeric bound -- already panics loudly here) but the resulting `Range` is mathematically
nonsensical, deferring the failure (or silent misbehavior) to whatever code eventually tries to use
`min`/`max` as actual bounds.

**Consumer audit:** No bundled chunk in the repo's `shader/` collection currently declares an
inverted range (confirmed via grep over all `//@ param:` lines) -- this is a manifest-authoring-time
guard against a future mistake, not a fix for a currently-manifesting bad chunk.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-defect sweep of the `shader_chunks_*` crate family, specifically
while auditing `range_clause_parse`'s validation completeness against its own established
panic-on-malformed convention (every other malformed shape already panics; the `min <= max`
relational check was the one shape silently accepted).

## Minimum Reproducible Example

```rust
// module/shader/shader_chunks_params_core/tests/discovery_test.rs
let _ = discover( "//@ param: x argument u32 range(8, 1)\n" );
// pre-fix: returns a Parameter with Range { min: 8.0, max: 1.0 }, no panic
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/shader/shader_chunks_params_core && cargo nextest run -E 'test(discover_panics_on_inverted_range_clause)'
```

## Root Cause

`range_clause_parse` parsed `min`/`max` as two independently-valid `f64` values with no relational
check between them. Each field was individually well-formed (a real, finite number), but nothing
checked that their combination formed a sensible range -- validating fields in isolation is not the
same as validating the value they jointly form.

## Why Not Caught

`discovery_test.rs`'s existing coverage of `range_clause_parse` exercised each malformed *shape*
(missing `range(...)`, missing comma, non-numeric min, non-numeric max) but never an inverted-but-
individually-valid pair, since no bundled chunk manifest happened to declare one.

## Fix Location

`module/shader/shader_chunks_params_core/src/lib.rs:265-291` (`range_clause_parse`): added
`assert!( min <= max, ... )` after both bounds are parsed, panicking with a message naming the
offending line, matching this function's own established panic-on-malformed convention for every
other shape.

## Prevention

New regression test `discover_panics_on_inverted_range_clause` (`discovery_test.rs`), marked
`#[should_panic(expected = "min must be <= max")]`, asserting `discover` on a `range(8, 1)` clause
panics instead of silently returning an inverted `Range`.

## Pitfall

A multi-field clause parsed as two independently-validated scalars can pass every per-field check
while still being jointly nonsensical -- per-field parsing is not a substitute for a cross-field
relational check when the fields together are supposed to form a single meaningful value (here, a
range). Any future clause with more than one field and an implied relationship between them (a
min/max pair, a start/end pair, an inclusive/exclusive pair) needs the same explicit cross-field
check added at parse time, not deferred to whatever downstream code eventually consumes the fields.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide `shader_chunks_*` bug/UX-defect sweep, auditing `range_clause_parse`'s validation completeness against its own panic-on-malformed convention. |
| 2026-08-20 | fixed | Added `min <= max` assertion to `range_clause_parse`, plus the 3-field `Fix(BUG-421)`/`Root cause`/`Pitfall` source comment. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily restored `range_clause_parse` to its pristine (pre-fix) form via `git show HEAD:` while keeping the new test in place -- confirmed the test fails ("test did not panic as expected"). Restored the fix -- test passes. Full scoped suite (`shader_chunks_render`, `shader_chunks_query`, `shader_chunks_query_core`, `shader_chunks_preview`, `shader_chunks_compose`, `shader_chunks_params`, `shader_chunks_params_core`): 147/147 pass; `cargo clippy` same scope, `--all-targets --all-features -- -D warnings`, clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-421)`/`Root cause`/`Pitfall` 3-field format applied at the fix site, matching this workspace's established source-comment convention (and the pre-existing `Fix(BUG-293)` comment style directly above it in the same file). | — |
| D3 | Scope containment | — | 🟢 | Fix touches only `range_clause_parse` in `shader_chunks_params_core/src/lib.rs`; no other function or crate modified for this bug. | — |

**Reproduced:** YES -- reverting `range_clause_parse` to its pristine form (via `git show
HEAD:module/shader/shader_chunks_params_core/src/lib.rs`, restored afterward) caused the new
`discover_panics_on_inverted_range_clause` test to fail ("test did not panic as expected"); restoring
the fix passes. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_params_core/src/lib.rs` | `range_clause_parse` (lines 265-291): added `assert!( min <= max, ... )` after parsing both bounds. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_params_core/tests/discovery_test.rs` | Added `discover_panics_on_inverted_range_clause` (`test_kind: bug_reproducer(BUG-421)`, `#[should_panic]`). |
