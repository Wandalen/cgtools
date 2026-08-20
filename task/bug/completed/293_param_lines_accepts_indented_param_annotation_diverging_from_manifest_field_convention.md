# BUG-293: `param_lines` recognizes an indented `//@ param:` line, diverging from every sibling manifest field's strict column-0 convention

- **Severity:** Low-Medium (no real bundled chunk triggers it today -- all 50 are flush-left by
  discipline -- but it's a live latent inconsistency: a future authored chunk with an indented
  illustrative `//@ param:` line inside a doc-comment example would be silently misparsed as real)
- **state:** Completed
- **Affects:** `param_lines` (`module/shader/shader_chunks_params_core/src/lib.rs`)
- **Component:** module/shader/shader_chunks_params_core
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

`param_lines` read lines via `line.trim_start().strip_prefix( "//@ param:" )`. Every sibling
header field -- `shader_chunks_core::manifest_field`/`manifest_field_opt`/`manifest_field_all`,
covering `name`/`description`/`tags`/`depends_on`/`export`/`stage` in the same flat `//@`-prefixed
block -- uses a bare `line.strip_prefix( prefix )` with no trim, requiring the prefix at column 0
strictly. `param_lines` silently diverged from that shared rule.

## Impact

**Who is affected:** any code discovering `//@ param:` lines from arbitrary/future WGSL where an
indented occurrence exists (e.g. an illustrative example inside a doc-comment block, indented to
match surrounding prose).

**What breaks:** this crate's own docs (`docs/api/001_tunable_parameter_taxonomy.md`'s Abstract)
explicitly claim `//@ param:` lives in "the same" header block under "the same trust model" as the
other 6 manifest fields -- a claim the leniency contradicted. A manifest system built on "malformed
authored content panics loudly" depends on every field sharing one predictable recognition rule; a
lone lenient field can silently accept content every other field would correctly ignore.

**Entity Scope:** `None` -- library text-parsing defect, not entity directory instances.

## How Discovered

Cross-checked `shader_chunks_params_core`'s claimed-convention docs against `manifest_field`'s
actual source line-for-line rather than trusting the shared vocabulary, during a systematic
bug-hunting pass across the 9 `module/shader/` crates with no prior recorded ledger investigation
(task #182).

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
cargo test -p shader_chunks_params_core --test discovery_test -- \
  param_line_requires_column_zero_prefix_matching_manifest_field_convention
```
**Expected** (fixed): 1 passed -- `discover` on a fixture with an indented `//@ param:` line
returns an empty parameter list. **Actual** (pre-fix): fails -- `discover` returned 1 parameter
instead of empty.

## Root Cause

`param_lines` was written independently of `manifest_field`/`manifest_field_all` rather than
mirroring their exact recognition rule, so it silently gained a leniency none of the other 6
manifest fields have.

## Why Not Caught

100% of real bundled chunk headers are flush-left by disciplined convention (verified: zero
indented `//@` lines of any kind across all 50 real chunk files), and every existing fixture in
`discovery_test.rs` already started `//@ param:` at column 0 -- nothing exercised the gap.

## Fix Applied (2026-08-18)

**`module/shader/shader_chunks_params_core/src/lib.rs`:** removed `.trim_start()` from
`param_lines`, so it now requires the `//@ param:` prefix at column 0, matching
`manifest_field`/`manifest_field_all`'s existing rule exactly. `Fix(BUG-293)`/Root cause/Pitfall
source comment added above the function.

**New regression test** (`tests/discovery_test.rs`):
`param_line_requires_column_zero_prefix_matching_manifest_field_convention` -- asserts `discover`
on a fixture containing an indented `//@ param:` line returns zero parameters.

## Verification

`longrun`-detached, from repo root, no `git stash` (RED proof obtained by testing against the
pre-fix source directly, before the fix was written).

- **Pre-fix (RED):** `cargo test -p shader_chunks_params_core --test discovery_test -- \
  param_line_requires_column_zero_prefix_matching_manifest_field_convention`: 0 passed; 1 failed.
- **Post-fix (GREEN):** same command: 1 passed. Combined scoped suite (`shader_chunks_params_core`
  + `shader_chunks_params` + `shader_chunks_cli_core` + `shader_chunks_preview`, run together with
  this bug's 2 siblings BUG-294/295/297): `48 tests run: 48 passed, 0 skipped`, clean clippy across
  all 4 crates -- independently re-run and confirmed by the orchestrating session, not only the
  investigating fork.

## Generalized Version

When a crate's docs claim it "mirrors" another module's convention, check the implementation
line-for-line against that module's actual source -- never assume consistency from shared
vocabulary alone. A `.trim_start()`/`.trim()` added "just in case" on one recognizer but not its
siblings is a common, easy-to-miss way for one field to silently diverge from an otherwise-uniform
parsing contract.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found by a fork investigating `shader_chunks_params`/`shader_chunks_params_core`/`shader_chunks_compose` (task #182, parallel with 2 sibling forks covering the other 6 of 9 previously-uninvestigated `module/shader/` crates). `shader_chunks_compose` itself was found clean -- its one real defect (duplicated named argument silently swallowed) was already fixed under pre-existing BUG-283 by an earlier pass. Fix and regression test written by the fork with a `BUG-XXX` placeholder (forks in this batch were instructed not to self-file, to avoid a 3-way concurrent-write race on the shared bug ledger); this report and its real ID were assigned by the orchestrating session after independently reading the actual committed diff and re-running the full scoped test suite. `task/readme.md`'s `highest_id` was found stale at 291 (task #292 had been filed by a concurrent, unrelated session actor without bumping it) immediately before filing -- corrected to 292 first, then this bug filed as 293, per a fresh on-disk scan across all `task/bug/` and `task/` lifecycle subdirectories. |
