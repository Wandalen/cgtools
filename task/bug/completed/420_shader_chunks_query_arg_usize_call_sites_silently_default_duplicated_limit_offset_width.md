# BUG-420: `shader_chunks_query`'s `limit`/`offset`/`width` call sites use unchecked `arg_usize`, silently defaulting a duplicated value to zero

- **Severity:** High (silently changes query results/formatting with no error, no warning, exit
  code 0 -- e.g. a duplicated `limit::` silently becomes "unlimited" instead of erroring)
- **state:** Completed
- **Affects:** `shader_chunks_query` binary (`list`/`get` commands) and any script/CI job driving it
  with `limit::`/`offset::`/`width::` built by string concatenation or repeated flag injection
- **Component:** `module/shader/shader_chunks_query` (`src/lib.rs`, `query_params_from`)
- **repo_identity:** self
- **Filed:** 2026-08-20
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-20
- **Related Bugs:** Direct follow-up to BUG-285 and BUG-295, not a duplicate of either --
  BUG-285 fixed this same function's `arg_string`/`arg_bool` call sites (17 of them) but its own
  "Generalized Version" section explicitly disclosed `arg_usize`'s identical catch-all shape as
  remaining unfixed "everywhere". BUG-295 then added `arg_usize_checked` to
  `shader_chunks_cli_core` as the fix vehicle, but its own "Prevention" section explicitly disclosed
  that `shader_chunks_query`'s 3 `arg_usize` call sites (`limit`, `offset`, `width`) were left
  "exposed to this defect until swapped to `arg_usize_checked`" as an out-of-scope, disclosed
  follow-up. This bug is that follow-up: migrating those 3 call sites to the already-existing
  `arg_usize_checked`, confirmed via a workspace-wide grep to be the last unchecked `arg_usize`
  callers.

## Symptom

```bash
shader_chunks_query list limit::5 limit::10
# pre-fix: exit 0, `limit` silently resolves to 0 ("unlimited") instead of erroring on the ambiguity
```

## Impact

**Who is affected:** Any caller of `list`/`get` that supplies `limit::`, `offset::`, or `width::`
twice -- most plausibly a script appending pagination arguments without checking whether one is
already present, or two independently-composed argument sources.

**What breaks:** The duplicated argument silently resolves to `0` (`arg_usize`'s catch-all default)
instead of erroring -- for `limit`/`offset` this means "unlimited"/"no skip" instead of either
requested value; for `width` it means "auto width" instead of either requested value. No error, no
warning, exit code 0.

**Consumer audit:** `query_params_from` is called from both `list` and `get`'s routines within this
same crate; a workspace-wide `grep -rn "arg_usize("` (excluding `arg_usize`'s own definition and
`arg_usize_checked`'s definition/call sites) confirms these were the only 3 remaining unchecked call
sites in the entire workspace.

**Entity Scope:** None -- a code-level defect.

## How Discovered

Found during a repo-wide bug/UX-defect sweep of the `shader_chunks_*` crate family, specifically
assigned as a dedup-check candidate against BUG-285. Investigation of BUG-285's and BUG-295's own
"Generalized Version"/"Prevention" sections confirmed this exact gap was identified by both prior
bugs but never actually closed -- explicitly named as disclosed follow-up work in both, not silently
missed.

## Minimum Reproducible Example

```rust
// module/shader/shader_chunks_query/tests/query_cli_test.rs
Command::cargo_bin( "shader_chunks_query" ).unwrap()
.args( [ "list", "limit::5", "limit::10" ] )
.output().unwrap();
// pre-fix: status code Some(0), limit silently resolves to 0 (unlimited)
```

**Verify Command** (<=3 lines, standalone):
```bash
cd module/shader/shader_chunks_query && cargo nextest run -E 'test(subprocess_list_with_duplicated_limit_fails_loudly_instead_of_defaulting_to_unlimited)'
```

## Root Cause

Same defect class as BUG-283/285/295: `shader_chunks_cli_core::arg_usize` matches
`cmd.arguments.get( name )` against `Some( Value::Integer )` with a bare `_ => Ok( 0 )` catch-all.
`unilang` binds ANY repeated named argument to `Value::List` regardless of the argument's own
declared `multiple` attribute, so a duplicated `limit::`/`offset::`/`width::` falls into the same
catch-all as a genuinely absent argument, silently resolving to `0` instead of surfacing the
ambiguity. `arg_usize_checked` (added by BUG-295) already closes this gap correctly -- these 3 call
sites simply hadn't been switched to it yet.

## Why Not Caught

BUG-295's own fix was scoped to adding `arg_usize_checked` to `shader_chunks_cli_core` without
migrating any caller (explicitly disclosed as out-of-scope follow-up in its own Prevention section);
no existing test in `shader_chunks_query` exercised a duplicated `limit::`/`offset::`/`width::`
before this bug's reproducer, since the crate's own call sites were never actually migrated.

## Fix Location

`module/shader/shader_chunks_query/src/lib.rs:119-134` (`query_params_from`): `limit`, `offset`, and
`width` switched from `arg_usize` to `arg_usize_checked`; import line updated accordingly.

## Prevention

New regression test `subprocess_list_with_duplicated_limit_fails_loudly_instead_of_defaulting_to_unlimited`
(`query_cli_test.rs`) drives the real subprocess with `limit::5 limit::10`, asserting exit code 1 and
a stderr message containing `` `limit` was given 2 times ``.

## Pitfall

A `_checked` sibling extractor only closes the defect gap for callers that actually switch to it --
adding the safer function is necessary but not sufficient; every existing call site of the unchecked
original remains exposed until migrated. When a fix's own scope is deliberately narrowed to "add the
tool" rather than "migrate every caller", the disclosed-but-unfixed callers need to be tracked as
concrete follow-up work (as both BUG-285 and BUG-295 did in their own text), not assumed closed by
the tool's mere existence.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-20 | filed | Found during repo-wide `shader_chunks_*` bug/UX-defect sweep; dedup-checked against BUG-285 and confirmed as the disclosed-but-unfixed follow-up named in both BUG-285's "Generalized Version" and BUG-295's "Prevention" sections. |
| 2026-08-20 | fixed | Migrated `limit`/`offset`/`width` call sites from `arg_usize` to `arg_usize_checked`, plus the 3-field `Fix(BUG-420)`/`Root cause`/`Pitfall` source comment. |
| 2026-08-20 | verified | See Verification Record below. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 3/3

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Regression test validity | — | 🟢 | Adversarial pass: temporarily restored `query_params_from` to its pristine (pre-fix) form via `git show HEAD:` while keeping the new test in place -- confirmed the test fails with `left: Some(0), right: Some(1)` (silent success instead of the expected error). Restored the fix -- test passes. Full scoped suite (`shader_chunks_render`, `shader_chunks_query`, `shader_chunks_query_core`, `shader_chunks_preview`, `shader_chunks_compose`, `shader_chunks_params`, `shader_chunks_params_core`): 147/147 pass; `cargo clippy` same scope, `--all-targets --all-features -- -D warnings`, clean. | — |
| D2 | Fix documentation compliance | — | 🟢 | `Fix(BUG-420)`/`Root cause`/`Pitfall` 3-field format applied at the fix site, matching this workspace's established source-comment convention (and BUG-285's own comment style directly above it). | — |
| D3 | Scope containment | — | 🟢 | Fix touches only the 3 named call sites plus the import line in `shader_chunks_query/src/lib.rs`; no other function or crate modified for this bug. | — |

**Reproduced:** YES -- reverting `query_params_from` to its pristine form (via `git show
HEAD:module/shader/shader_chunks_query/src/lib.rs`, restored afterward) caused the new
`subprocess_list_with_duplicated_limit_fails_loudly_instead_of_defaulting_to_unlimited` test to fail
(`left: Some(0), right: Some(1)`, i.e. silent exit-0 success instead of the expected exit-1 error);
restoring the fix passes. 2026-08-20.

## Refs: src/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_query/src/lib.rs` | `query_params_from` (lines 119-134): `limit`/`offset`/`width` switched from `arg_usize` to `arg_usize_checked`; import updated. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_query/tests/query_cli_test.rs` | Added `subprocess_list_with_duplicated_limit_fails_loudly_instead_of_defaulting_to_unlimited` (`test_kind: bug_reproducer(BUG-420)`). |
