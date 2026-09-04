# BUG-285: `shader_chunks_query`/`shader_chunks_render`/`shader_chunks_preview` still call the unchecked `arg_string`/`arg_bool` helpers, silently dropping a duplicated named argument in all three CLIs

- **Severity:** Medium (no data corruption on disk, but a silent behavior divergence from an
  explicit user request in three separate CLIs -- e.g. `render fbm3 out::a.png out::b.png` reports
  success (exit 0) and writes to neither requested path, silently reusing the default instead)
- **state:** Completed
- **Affects:** `shader_chunks_query::query_params_from` + `cmd_tree`'s routine (`src/lib.rs`);
  `shader_chunks_render`'s `render` routine (`src/lib.rs`); `shader_chunks_preview`'s `preview`
  routine (`src/lib.rs`) -- all via `shader_chunks_cli_core::arg_string`/`arg_bool`
- **Component:** `module/shader/shader_chunks_query` + `module/shader/shader_chunks_render` +
  `module/shader/shader_chunks_preview`
- **repo_identity:** self
- **Filed:** 2026-08-17
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-17
- **Fixed:** 2026-08-17
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

BUG-283 fixed this same defect class in `shader_chunks_cli_core`'s `arg_string`/`arg_bool` catch-all
arms, but applied the fix only to `shader_chunks_compose`'s 2 call sites -- its own report's
"Generalized Version" section explicitly named `shader_chunks_preview`, `shader_chunks_query`, and
`shader_chunks_render` as still calling the unchecked originals, "outside this fork's assigned
scope." All three remained genuinely vulnerable: any named `key::value` argument repeated on argv
silently falls back to its default instead of erroring, across 26 call sites total --
`shader_chunks_query`'s `list`/`get`/`tree` commands (17 sites: `pattern`, `case`, `tags_mode`,
`stage`, `depends_on`, `transitive`, `exports`, `source`, `roots`, `leaves`, `count`, `format`,
`sort`, `order`, `heading` in `query_params_from`; `reverse`, `shape` in `cmd_tree`),
`shader_chunks_render`'s `render` command (7 sites: `name`, `file`, `all`, `size`×2, `out`×2), and
`shader_chunks_preview`'s `preview` command (3 sites: `name`, `file`, `serve`).

## Impact

**Who is affected:** any user or script that accidentally repeats a named `key::value` argument on
a `shader_chunks_query`/`shader_chunks_render`/`shader_chunks_preview` invocation (or their
aggregated `sch` spellings) -- shell history editing, copy-paste, or a generated command line that
appends a default before a user-supplied override, same plausible-mistake shape BUG-283 already
documented for `compose`.

**What breaks:** the CLI family's own "loud errors, no silent defaults" convention (already fixed
for `compose` under BUG-283). Concretely: `list pattern::a pattern::b` silently matches every chunk
instead of filtering (an empty pattern is the default); `render fbm3 out::a.png out::b.png` silently
writes `fbm3.png` instead of erroring, discarding both paths the caller actually typed; `preview
fbm3 file::a.wgsl file::b.wgsl serve::0` silently ignores the duplicated `file::` and previews `fbm3`
by name instead. In every case the command exits 0 and gives no indication that a supplied argument
was ignored.

**Entity Scope:** `None` -- CLI argument-handling defect in library code, not entity directory
instances.

## How Discovered

While closing out this session's `module/shader` bug-hunt task (6 dispatched forks found BUG-280
through BUG-284), ran an adversarial spot-check of BUG-283's own stated scope boundary before
accepting its fix as complete: grepped every crate in `module/shader` for direct
`shader_chunks_cli_core::arg_string`/`arg_bool` usage rather than the new `_checked` variants.
`shader_chunks_query`, `shader_chunks_render`, and `shader_chunks_preview` all still called the
unchecked originals, exactly as BUG-283's own "Generalized Version" section had already flagged as
a known, deliberately out-of-scope gap for that fork -- confirming a real, live, in-scope defect
rather than a hypothetical one.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools
./target/debug/shader_chunks_render render fbm3 out::/tmp/dupA.png out::/tmp/dupB.png; echo "exit=$?"
ls /tmp/dupA.png /tmp/dupB.png fbm3.png 2>&1
```
**Expected** (fixed): exits 1, stderr names the offending `out` parameter, no PNG is written anywhere.
**Actual** (pre-fix): exits 0, writes `fbm3.png` (the default derived path) in the current directory,
`/tmp/dupA.png`/`/tmp/dupB.png` are never created -- both explicitly requested paths silently
discarded.

## Root Cause

Identical to BUG-283's own Root Cause section: `unilang::semantic::argument_binding::bind_argument_values`
binds *any* repeated named argument to `Value::List` unconditionally, before ever consulting the
argument's own declared `multiple` attribute. `shader_chunks_cli_core::arg_string`/`arg_bool`'s
catch-all `_` arms treat that `Value::List` identically to "the key was never supplied" -- BUG-283
fixed this by adding `arg_string_checked`/`arg_bool_checked` (explicit `Value::List` arm, loud
`ValidationRuleFailed` error), but only rewired `shader_chunks_compose`'s 2 call sites to use them,
leaving every other CLI crate's call sites on the original unchecked helpers.

## Why Not Caught

BUG-283's fix was scoped to the single fork's assigned crate pair; its own report named the
remaining 3 crates as an explicit, acknowledged gap rather than an oversight, so nothing was
"missed" so much as deferred and not yet separately tracked. No existing test in any of the 3
crates fixed here ever repeated a named `key::value` argument in one invocation.

## Fix Applied (2026-08-17)

**`shader_chunks_query/src/lib.rs`:** all 15 call sites in `query_params_from` and 2 in `cmd_tree`'s
routine switched from `arg_string`/`arg_bool` to `arg_string_checked`/`arg_bool_checked` (via `?`);
import list updated accordingly. `arg_list`-based fields (`names`, `tag`, `fields`) are untouched --
those are genuinely `multiple: true`/comma-delimited arguments by design, not vulnerable to this
defect class.

**`shader_chunks_render/src/lib.rs`:** all 7 call sites in the `render` routine (`name`, `file`,
`all`, both `size` reads, both `out` reads) switched the same way.

**`shader_chunks_preview/src/lib.rs`:** all 3 call sites in the `preview` routine (`name`, `file`,
`serve`) switched the same way.

**New regression tests** (3 files, 5 new tests total):
- `shader_chunks_query/tests/query_cli_test.rs` (new file -- this crate had no `tests/*.rs` at all
  before this fix): `subprocess_list_with_duplicated_pattern_fails_loudly_instead_of_matching_everything`,
  `subprocess_tree_with_duplicated_reverse_fails_loudly`, plus a same-file guard test
  (`subprocess_list_with_single_pattern_still_succeeds`) proving the fix doesn't over-reject a
  single occurrence.
- `shader_chunks_render/tests/render_cli_test.rs`:
  `subprocess_render_with_duplicated_out_fails_loudly_instead_of_using_the_default_path`.
- `shader_chunks_preview/tests/preview_cli_test.rs`:
  `subprocess_preview_with_duplicated_file_fails_loudly_instead_of_falling_back_to_the_name_target`.

## Verification

`longrun`-detached, from repo root. Revert-and-rerun proof used a scratchpad copy of the 3 fixed
files plus `git show HEAD:<path>` to temporarily restore pristine content -- never `git stash`, per
this session's own confirmed concurrent-fork stash-collision hazard (not actually a concurrency risk
for this sequential, non-forked fix, but kept as standing practice regardless).

- **Pre-fix (RED):** `cargo nextest run -p shader_chunks_query -p shader_chunks_render
  -p shader_chunks_preview --test query_cli_test --test render_cli_test --test preview_cli_test -E
  'test(duplicated)'` against the temporarily-restored pristine source: all 4 new tests failed as
  predicted (`0 passed; 1 failed` each), confirming each bug before any fix existed.
- **Pre-fix test-design incident (self-caught, corrected before filing):** the first draft of the
  `preview` regression test duplicated `serve::0 serve::1` rather than `file::`. Against the
  pristine pre-fix source this resolved to the silently-defaulted `serve = true` and actually
  launched a real `trunk serve` dev server plus a real Firefox window (`action/browser_serve`) --
  the test's own 30s `assert_cmd` process timeout killed the immediate `shader_chunks_preview`
  child, but `trunk serve`/Firefox are *grandchildren*, inherited the same stdout/stderr pipes, and
  kept them open after the immediate child died, hanging the whole `cargo nextest` run's pipe-EOF
  read past 340s wall-clock. Identified the exact orphaned PIDs via `ps` (bash `action/browser_serve`
  wrapper, `trunk serve --release --port 38613`, Firefox parent) -- unambiguously test-spawned by
  working directory and port, distinct from the user's own pre-existing, unrelated long-running
  browser session -- and terminated only those 3 PIDs (`kill`, no pattern-based `pkill`), which
  unblocked the hung nextest run immediately. Rewrote the test to duplicate `file::` instead (an
  inert named string, no side effects) with `serve::0` pinned unambiguously on both invocations, so
  the duplication under test can never reach the serve branch regardless of fix state; re-ran the
  corrected pre-fix (RED, 0.03s, no hang) and post-fix (GREEN) cases cleanly.
- **Post-fix (GREEN):** `cargo nextest run -p shader_chunks_query -p shader_chunks_render
  -p shader_chunks_preview` (full suite, all 3 crates) + `cargo clippy -p shader_chunks_query
  -p shader_chunks_render -p shader_chunks_preview --all-targets --all-features -- -D warnings`,
  chained: 52 passed / 0 failed, zero clippy warnings/errors.

## Generalized Version

Same broken assumption as BUG-283: a `Value` match's catch-all `_` arm cannot distinguish "argument
absent" from "argument supplied redundantly," and `unilang` collapses any repeated named key to
`Value::List` regardless of its own declared `multiple` attribute. A fix for this defect class that
adds a new checked helper without also migrating every existing call site leaves the original,
silently-wrong helper just as reachable as before -- `grep` for the unchecked helper's name across
sibling crates is the concrete way to confirm a "fixed" defect class is actually fully migrated, not
just fixed at the one call site that happened to be investigated. `shader_chunks_cli_core::arg_usize`
(`limit`/`offset`/`width` in `shader_chunks_query`, unused directly by `render`/`preview`) has the
identical catch-all shape (`_ => Ok(0)`) and remains unfixed everywhere -- consistent with BUG-283's
own stated scope boundary, left as a known, not forgotten, gap rather than expanded into a fourth
helper mid-fix.

Separately: a boolean CLI argument whose default is "silently starts a real, long-lived child
process" (here, `serve`, default `true`) turns *any* test that lets its value resolve ambiguously --
not just a deliberately malformed one -- into a live-side-effect risk. `assert_cmd`'s own process
timeout only guarantees the *immediate* child is killed; it does not reach grandchildren that
inherited the same stdout/stderr pipe and are still alive, so a killed-on-timeout test can still hang
the whole test runner indefinitely on pipe EOF. Any test touching such an argument should pin its
safe value explicitly and never rely on an edge-case input coincidentally resolving away from the
dangerous default.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-17 | filed + fixed + verified | Found via an adversarial spot-check of BUG-283's own stated scope boundary (that fork's report explicitly named `shader_chunks_query`/`shader_chunks_render`/`shader_chunks_preview` as still vulnerable) during this session's `module/shader` bug-hunt task closeout. Same root cause as BUG-283: `unilang` binds any repeated named argument to `Value::List` regardless of its own `multiple` attribute; `arg_string`/`arg_bool`'s catch-all arms silently treat that as "argument absent." Fixed by switching all 26 remaining call sites across the 3 crates to the already-existing `arg_string_checked`/`arg_bool_checked` helpers. Verified via 5 new regression tests, confirmed failing against a temporarily-restored pristine source first (scratchpad copy + `git show HEAD:<path>`, no `git stash`) then passing post-fix, plus the full 3-crate suite (52/52) and clean clippy. One test-design mistake self-caught mid-verification: an initial `preview` regression test duplicated `serve::` and accidentally launched a real browser+dev-server process tree against the pre-fix source, hanging the test run past 340s via an orphaned-grandchild-holds-the-pipe-open mechanism; identified and terminated the exact 3 spawned PIDs via `ps` (no pattern-based kill), then redesigned the test to duplicate the inert `file::` argument instead. `task/readme.md`'s `highest_id` stood at 284 at filing time, confirmed via a fresh on-disk scan immediately before filing. |
