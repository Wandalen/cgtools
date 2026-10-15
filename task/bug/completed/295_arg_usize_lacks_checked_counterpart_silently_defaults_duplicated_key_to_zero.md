# BUG-295: `arg_usize` lacks a duplicate-detecting `_checked` counterpart, same defect class as BUG-283/285

- **Severity:** Medium (silent default-instead-of-error, same class as BUG-283/BUG-285)
- **state:** Completed
- **Affects:** `arg_usize` callers across the `shader_chunks_*` CLI family (currently
  `shader_chunks_query`'s `limit`/`offset`/`width` -- out of this bug-hunt's scope; the extractor
  itself lives in `shader_chunks_cli_core`)
- **Component:** module/shader/shader_chunks_cli_core
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`arg_usize`'s catch-all `_ => Ok( 0 )` arm can't tell "argument absent" from "argument supplied
twice" -- a duplicated integer-valued key silently resolved to `0` instead of erroring.

## Impact

**Who is affected:** any command using `arg_usize` on a duplicated named argument.

**What breaks:** the caller silently uses the wrong value (an unbounded/zero default) instead of
failing loudly -- e.g. `sch list limit::5 limit::10` would silently resolve `limit` to `0`
(unlimited) instead of erroring, exactly the class of surprise BUG-283/285 eliminated for
`arg_string`/`arg_bool`.

**Entity Scope:** `None` -- library argument-parsing defect.

## How Discovered

BUG-283's and BUG-285's own Pitfall comments (`shader_chunks_query/src/lib.rs`,
`shader_chunks_preview/tests/preview_cli_test.rs`) explicitly named `arg_usize` as "a known, not a
forgotten, gap" -- followed up during exhaustive review of `shader_chunks_cli_core/src/lib.rs`
(task #182's bug-hunting pass).

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p shader_chunks_cli_core --test shader_chunks_cli_core_test -- \
  arg_usize_checked_fails_loudly_on_duplicated_value_instead_of_defaulting_to_zero
```
**Expected** (fixed): 1 passed. **Actual** (against the old `arg_usize` alone, pre-`_checked`):
a `Pipeline` dispatch of a throwaway command with a duplicated integer argument resolves it to `0`
instead of erroring.

## Root Cause

`unilang` binds ANY repeated named argument to `Value::List` regardless of the argument's own
`multiple` attribute; a bare `_` catch-all over `Value` cannot distinguish absent from duplicated.

## Why Not Caught

BUG-283/285 fixed every `arg_string`/`arg_bool` call site but explicitly left `arg_usize` itself
unfixed as disclosed follow-up debt (see both bugs' own Pitfall sections).

## Fix Applied (2026-08-18)

Added `arg_usize_checked` (additive, mirrors `arg_bool_checked`'s shape exactly: matches
`Value::Integer` for the normal case, `Value::List` for an explicit loud error naming the key and
repeat count, and the same catch-all default as `arg_usize` for the absent case). `arg_usize`
itself is untouched -- unchecked existing callers keep their current behavior, matching how
`arg_string`/`arg_bool` were migrated (additive, not a breaking in-place change). `Fix(BUG-295)`/
Root cause/Pitfall source comment added above the new function; exported via `mod_interface!`.

**New regression test** (`tests/shader_chunks_cli_core_test.rs`):
`arg_usize_checked_fails_loudly_on_duplicated_value_instead_of_defaulting_to_zero`, using a new
`int_arg_command` test helper.

## Verification

`longrun`-detached, from repo root, no `git stash`.

- **Pre-fix (RED):** `cargo test -p shader_chunks_cli_core --test shader_chunks_cli_core_test --
  arg_usize_checked_fails_loudly_on_duplicated_value_instead_of_defaulting_to_zero`: 0 passed;
  1 failed.
- **Post-fix (GREEN):** same command: 1 passed. Full crate suite: `3 tests run: 3 passed,
  0 skipped`, clean clippy. Wider combined scoped suite (with sibling BUG-293/294/297, run
  together): `48 tests run: 48 passed, 0 skipped`, clean clippy across all 4 crates --
  independently re-run and confirmed by the orchestrating session, not only the investigating fork.

## Generalized Version

Any single-value extractor built on a bare `_` catch-all over `unilang::Value` needs an explicit
`Value::List` arm -- silent default-on-duplicate is invisible until someone types the argument
twice. When one extractor in a family (`arg_string`, `arg_bool`) gets this fix, audit every sibling
extractor built the same way rather than assuming the fix generalized automatically.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found by a fork investigating `shader_chunks_cli_core`/`shader_chunks_preview`/`shader_chunks_preview_web` (task #182, parallel with 2 sibling forks). Fix and regression test written by the fork with a `BUG-XXX` placeholder (forks in this batch were instructed not to self-file, to avoid a 3-way concurrent-write race on the shared bug ledger); this report and its real ID were assigned by the orchestrating session after independently reading the actual committed diff and re-running the full scoped test suite. |
