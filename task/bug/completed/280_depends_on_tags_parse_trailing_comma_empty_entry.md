# BUG-280: `depends_on_parse`/`tags_parse` (and `build.rs`'s `list_entries`) keep a spurious empty entry from a trailing/doubled comma in a `//@` manifest list

- **Severity:** Low (no defect against any of the 50 currently-bundled `shader/*/*.wgsl` manifests
  -- a latent parsing-robustness defect in the public `depends_on_parse`/`tags_parse` API and the
  `build.rs` compile-time table generator, reachable by any comma-list manifest value with a
  trailing, leading, or doubled comma)
- **state:** Completed
- **Affects:** `shader_chunks_core::depends_on_parse`, `shader_chunks_core::tags_parse`
  (`src/lib.rs`), and `build.rs`'s `list_entries` (the compile-time counterpart shared by both the
  `depends_on` and `tags` fields when generating `CHUNKS`)
- **Component:** `module/shader/shader_chunks_core`
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox tsk actor-guard blocks .acceptance_pass in this environment)

## Symptom

`depends_on_parse` and `tags_parse` read a `//@ depends_on:`/`//@ tags:` manifest value and split
it into entries on `,`, trimming each piece. Neither step then drops a piece that trims down to an
empty string. A trailing comma (`//@ depends_on: a,`), a leading comma, or a doubled comma
(`a,,b`) all produce one extra `""` entry alongside the real names. `depends_on_parse` returns that
empty string silently, as if it were a real chunk name; `tags_parse` instead panics on it via
`split_once(':')`'s `None` arm, with a "malformed `//@ tags:` entry" message that doesn't point at
the actual cause (a stray comma, not an unpaired `group:tag`). `build.rs`'s `list_entries` --
generating the compile-time `CHUNKS` table's `depends_on`/`tags` fields from the identical
`split(',').map(str::trim)` pattern -- carried the same defect.

## Impact

**Who is affected:** any caller of the public `depends_on_parse`/`tags_parse` functions passing
manifest text with a stray comma in a `depends_on`/`tags` list -- e.g. a chunk author who leaves a
trailing comma after editing a dependency list, or any external tool built against this crate's
public parsing API.

**What breaks:** `depends_on_parse` silently returns an extra `""` "dependency" name mixed into an
otherwise-correct list -- violating its own documented contract ("a list of chunk names"), since an
empty string is not a chunk name. If that bogus entry ever reached `compose`/`try_compose`, it
would surface as a confusing `ComposeError::MissingDependency { missing: "" }` instead of a message
naming the actual stray-comma typo. `tags_parse` panics instead, with a message that doesn't name
the real cause. None of the 50 currently-bundled chunk manifests trigger this (confirmed by
`grep`), so no shipped chunk is affected today; this is a latent robustness gap in the parsing
functions themselves.

**Entity Scope:** None -- Rust parsing-logic defect, not entity directory instances.

## How Discovered

During this session's full-file review of `shader_chunks_core` (assigned crate pair
`shader_chunks_core` + `shader_chunks`), per the domain hint to specifically probe "string-parsing
bugs in the `//@` manifest comment extraction (what happens with extra whitespace, a missing
trailing newline, or a chunk with zero dependencies?)". The zero-dependency and pure-whitespace
cases were already correctly handled and covered by existing tests
(`parse_depends_on_handles_empty_value`); tracing `depends_on_parse`'s `raw.split(',').map(
str::trim ).collect()` pipeline by hand against a value ending in a comma showed `split` yields a
trailing empty segment that trimming does not remove. Independently re-verified by locating the
identical unfiltered pattern in `tags_parse` and in `build.rs`'s `list_entries` (the crate's own
module doc states `list_entries` deliberately mirrors the `lib.rs` parsers so the two "cannot
silently drift" -- confirming this was one shared root cause in three call sites, not three
unrelated bugs) before filing.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p shader_chunks_core --test shader_chunks_core_test stray_commas
```
**Expected** (fixed): both `parse_depends_on_ignores_stray_commas` and
`parse_tags_ignores_stray_commas` pass.
**Actual** (pre-fix):
```
---- parse_depends_on_ignores_stray_commas stdout ----
assertion `left == right` failed
  left: ["a", "b", ""]
 right: ["a", "b"]

---- parse_tags_ignores_stray_commas stdout ----
thread panicked at shader_chunks_core/src/lib.rs:...
malformed `//@ tags:` entry (expected `group:tag`): ""
```

## Root Cause

`src/lib.rs` (pre-fix):
```rust
pub fn depends_on_parse( wgsl : &str ) -> Vec< &str >
{
  let raw = manifest_field( wgsl, "depends_on" );
  if raw.is_empty() { return Vec::new(); }
  raw.split( ',' ).map( str::trim ).collect()
}
```
`"a, b,".split(',')` yields `["a", " b", ""]` -- one segment per delimiter occurrence, including
the empty tail after the trailing comma. `str::trim` trims whitespace but does not remove an
already-empty string, so `""` survives into the collected `Vec`. `tags_parse` and `build.rs`'s
`list_entries` shared the identical `split(',').map(str::trim)` step with no filter, so the same
artifact reached both the `tags` field and the compile-time-generated `CHUNKS` table.

## Why Not Caught

Existing coverage (`parse_depends_on_handles_empty_value`, `parse_depends_on_handles_multiple_entries`)
exercised only a fully-empty value and a clean `"a, b"` two-entry list -- never a list with a
trailing/doubled-comma artifact -- and none of the 50 bundled `shader/*/*.wgsl` manifests happen to
have one (confirmed via `grep -rn "depends_on:.*,\s*$" shader/*/*.wgsl`, zero hits), so the defect
never surfaced against real registry data.

## Fix Applied

`depends_on_parse` and `tags_parse` (`src/lib.rs`) and `list_entries` (`build.rs`) now add
`.filter( | entry | !entry.is_empty() )` to the same `split(',').map(str::trim)` pipeline, dropping
empty segments before they are collected (`depends_on_parse`) or paired via `split_once(':')`
(`tags_parse`). A genuinely malformed non-empty entry (no `:` separator) still panics in
`tags_parse`, unchanged -- only the comma-artifact empty case is now tolerated, the same way
surrounding whitespace already was.

## Verification

`longrun`-detached, from repo root, invoked from an isolated scratchpad cwd to avoid Durable-Log
filename collisions with other concurrently-running forks sharing this same `shader/` directory:
- **Pre-fix (RED):** test added first, source not yet touched --
  `cargo test -p shader_chunks_core --test shader_chunks_core_test stray_commas`: both new tests
  fail (`parse_depends_on_ignores_stray_commas` on the `["a","b",""]` vs `["a","b"]` mismatch;
  `parse_tags_ignores_stray_commas` on the "malformed" panic) -- exit 101, 2 failed.
- **Post-fix (GREEN):** same command: `test parse_depends_on_ignores_stray_commas ... ok`,
  `test parse_tags_ignores_stray_commas ... ok` -- 2 passed, 0 failed.
- Full scoped suite: `cargo test -p shader_chunks_core -p shader_chunks --all-features`: every
  binary reports `test result: ok` with 0 failed (37/37 in `shader_chunks_core_test`, including the
  2 new tests, plus the crate's `compile_fail_test` trybuild suite, doctests, and the full
  `shader_chunks` aggregator + CLI-subprocess suite) -- 0 failures anywhere.
- `cargo clippy -p shader_chunks_core -p shader_chunks --all-targets --all-features -- -D
  warnings`: clean, exit 0 (`Finished` with no `warning:`/`error:` lines attributable to this
  change; the one unrelated `warning: patch core2 ... was not used in the crate graph` is a
  pre-existing workspace-manifest note, not a lint on this code).

## Generalized Version

**Broken assumption:** a `str::split(',').map(str::trim)` pipeline over a human-authored,
comma-separated value assumes every yielded segment is a real entry. `split` always yields exactly
`delimiter_count + 1` segments, so a trailing, leading, or doubled delimiter always produces at
least one segment that trims down to empty -- `trim` normalizes whitespace, it does not fill the
"list is well-formed" gap. Any comma-list (or similar delimiter-list) parser must add an explicit
`filter(|s| !s.is_empty())` (or equivalent) step, and when the same textual grammar is
deliberately re-implemented in two places for a good reason (here: a `build.rs` script cannot
depend on the crate it is generating code for), both implementations need the fix applied
identically, not just the one first noticed.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found during this session's full-file review of `shader_chunks_core` + `shader_chunks` (10 assigned files), prompted by the domain hint to probe `//@` manifest whitespace/parsing edge cases. Root cause: `depends_on_parse`, `tags_parse` (`src/lib.rs`), and `list_entries` (`build.rs`, shared by both fields at compile time) all split a comma-separated manifest value and trimmed each piece without filtering empty segments, so a trailing/leading/doubled comma silently produced a spurious `""` entry (`depends_on_parse`) or a misleading panic (`tags_parse`). Fixed by adding `.filter(|entry| !entry.is_empty())` to all three call sites. Verified via 2 new native unit tests (confirmed fail pre-fix / pass post-fix, tests written and RED-checked before the fix was applied, no revert needed) plus the full `--all-features` suite across both assigned crates and clean clippy. Fresh on-disk scan immediately before filing confirmed 279 was still the true max (`task/readme.md`'s `highest_id` also read 279), so filed directly as BUG-280 with no renumbering needed. |
