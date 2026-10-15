# BUG-419: `shader_chunks_render`'s `arg_time` silently drops a duplicated `time::` argument instead of erroring

- **Severity:** High (silently renders the wrong frame -- no error, no warning, exit code 0 -- with a
  PNG written using `0.0` instead of either of the two values the caller actually gave)
- **state:** Completed
- **Affects:** `shader_chunks_render` binary and any script/CI job driving it with a `time::` argument
  built by string concatenation or repeated flag injection (the exact shape that produces an
  accidental duplicate)
- **Component:** `module/shader/shader_chunks_render` (`src/lib.rs`, `arg_time`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-20
- **Related Bugs:** Same defect class as BUG-283/285/295 (`unilang` binds ANY repeated named
  argument to `Value::List` regardless of the argument's own declared `multiple` attribute, and a
  bare `_` catch-all over `Value` cannot distinguish "argument absent" from "argument supplied
  twice") -- those bugs fixed `shader_chunks_compose`/`shader_chunks_query`/`shader_chunks_render`'s
  *shared* `arg_string`/`arg_bool`/`arg_usize` extractors in `shader_chunks_cli_core`. `arg_time` is
  a crate-local helper unique to `shader_chunks_render` (not one of the shared extractors), so it
  was never touched by those fixes and carried the identical defect shape independently.

## Symptom

```bash
shader_chunks_render render fbm3 out::frame.png time::1.0 time::2.0
# pre-fix: exit 0, frame.png written using time = 0.0 (neither 1.0 nor 2.0, no error at all)
```

## Impact

**Who is affected:** Any caller of the `render` command that supplies `time::` twice -- most
plausibly a script that appends `time::<value>` to an argument list without checking whether one is
already present, or a CI matrix that composes arguments from two independent sources.

**What breaks:** The written PNG silently uses `time = 0.0`, which is neither of the two values
given and is not flagged as an error in any way -- exit code 0, no stderr output. A caller inspecting
only the exit code (the normal automation pattern) has no signal anything went wrong; the defect is
only visible by manually inspecting the rendered frame's content against the intended time.

**Consumer audit:** `arg_time` has exactly one call site, `cmd_render`'s own routine
(`shader_chunks_render/src/lib.rs`) -- the fix is fully contained to this one crate, no other crate
calls this function.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-defect sweep of the `shader_chunks_*` crate family, specifically
triaged as a follow-up to the `arg_string`/`arg_bool`/`arg_usize` catch-all family already fixed by
BUG-283/285/295 -- `arg_time` was checked as a candidate sibling with the same shape and confirmed to
share the identical `_ => 0.0` catch-all arm those bugs already established as defective.

## Minimum Reproducible Example

```rust
// module/shader/shader_chunks_render/tests/render_cli_test.rs
Command::cargo_bin( "shader_chunks_render" ).unwrap()
.args( [ "render", "fbm3", "out::<tmp>.png", "time::1.0", "time::2.0" ] )
.output().unwrap();
// pre-fix: status code Some(0), <tmp>.png written using time = 0.0
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/shader/shader_chunks_render && cargo nextest run -E 'test(subprocess_render_with_duplicated_time_fails_loudly_instead_of_defaulting_to_zero)'
```

## Root Cause

`arg_time` matched `cmd.arguments.get( "time" )` against `Some( Value::Float )` /
`Some( Value::Integer )` with a bare `_ => 0.0` catch-all for everything else. `unilang` binds ANY
repeated named argument -- regardless of that argument's own declared `multiple` attribute -- to
`Value::List` rather than rejecting the duplicate or keeping only the last value. Since `arg_time`
had no `Value::List` arm, a duplicated `time::` fell into the same catch-all as a genuinely absent
argument, silently resolving to the `0.0` default instead of surfacing the ambiguity.

## Why Not Caught

`arg_time` is a crate-local helper written independently of `shader_chunks_cli_core`'s shared
`arg_*`/`arg_*_checked` extractors, so BUG-283/285/295's fixes -- which only touched call sites of
those shared functions -- never reached it; nothing in this crate's existing test suite exercised a
duplicated `time::` argument before this bug's reproducer.

## Fix Location

`module/shader/shader_chunks_render/src/lib.rs:324-343` (`arg_time`): added an explicit
`Value::List` arm that returns `Err` with a `ValidationRuleFailed` error naming how many times
`time` was given, instead of falling through to the `_ => 0.0` default.

## Prevention

New regression test `subprocess_render_with_duplicated_time_fails_loudly_instead_of_defaulting_to_zero`
(`render_cli_test.rs`) drives the real subprocess with `time::1.0 time::2.0`, asserting exit code 1,
a stderr message containing `` `time` was given 2 times ``, and that no PNG is written on failure.

## Pitfall

A crate-local single-value argument extractor written by hand (rather than reusing a shared
`arg_*_checked` helper) silently reintroduces this exact defect class unless it explicitly handles
`Value::List` from the start -- a bare `_` catch-all over `Value` is the recurring failure shape
across BUG-283/285/295/419 alike. Any future crate-local extractor built the same way needs the
identical `Value::List` arm, not a bare `_` catch-all, from the moment it's written.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide `shader_chunks_*` bug/UX-defect sweep, triaged as a sibling of the `arg_string`/`arg_bool`/`arg_usize` catch-all family (BUG-283/285/295). |
| 2026-08-20 | fixed | Added explicit `Value::List` rejection arm to `arg_time`, plus the 3-field `Fix(BUG-419)`/`Root cause`/`Pitfall` source comment. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily restored `arg_time` to its pristine (pre-fix) form via `git show HEAD:` while keeping the new test in place -- confirmed the test fails with `left: Some(0), right: Some(1)` (silent success instead of the expected error). Restored the fix -- test passes. Full scoped suite (`shader_chunks_render`, `shader_chunks_query`, `shader_chunks_query_core`, `shader_chunks_preview`, `shader_chunks_compose`, `shader_chunks_params`, `shader_chunks_params_core`): 147/147 pass; `cargo clippy` same scope, `--all-targets --all-features -- -D warnings`, clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-419)`/`Root cause`/`Pitfall` 3-field format applied at the fix site, matching this workspace's established source-comment convention. | — |
| D3 | Scope containment | — | 🟢 | Fix touches only `arg_time` in `shader_chunks_render/src/lib.rs`; no other function or crate modified for this bug. | — |

**Reproduced:** YES -- reverting `arg_time` to its pristine form (via `git show HEAD:module/shader/shader_chunks_render/src/lib.rs`, restored afterward) caused the new
`subprocess_render_with_duplicated_time_fails_loudly_instead_of_defaulting_to_zero` test to fail
(`left: Some(0), right: Some(1)`, i.e. silent exit-0 success instead of the expected exit-1 error);
restoring the fix passes. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_render/src/lib.rs` | `arg_time` (lines 324-357): added a `Value::List` arm that errors loudly on a duplicated `time::` argument instead of falling through to the `0.0` default. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_render/tests/render_cli_test.rs` | Added `subprocess_render_with_duplicated_time_fails_loudly_instead_of_defaulting_to_zero` (`test_kind: bug_reproducer(BUG-419)`). |
