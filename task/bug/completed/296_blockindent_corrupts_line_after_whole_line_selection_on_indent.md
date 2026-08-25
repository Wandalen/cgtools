# BUG-296: `blockIndent()` corrupts the line *after* a whole-line selection when indenting

- **Severity:** Medium (silent data corruption in a user-facing editor widget, not merely a
  cosmetic glitch -- indenting a selected block silently mutates unrelated adjacent text)
- **state:** Completed
- **Affects:** `blockIndent()` (`module/shader/shader_chunks_preview_web/controls.js`)
- **Component:** module/shader/shader_chunks_preview_web
- **repo_identity:** self
- **Filed:** 2026-08-18
- **filed_by:** self
- **verified_by:** self
- **verification_date:** 2026-08-18
- **Fixed:** 2026-08-18
- **Accepted By:** self -- same-session Tier 2 Dual-Role Self-Check, no separate acceptance actor
  (same-sandbox `tsk` actor-guard blocks `.acceptance_pass` in this environment)

## Symptom

Selecting one or more whole lines (e.g. a triple-click, or any selection whose end lands exactly
at the start of the following line) and pressing Tab to indent silently added 2 spaces to the
start of the line *after* the selection, in addition to (or instead of, depending on exact
selection shape) the intended lines.

## Impact

**Who is affected:** any user of the in-browser WGSL editor (`shader_chunks_preview_web`) who
selects whole lines by triple-click or similar and indents them.

**What breaks:** silent corruption of text the user never selected -- the kind of editor defect
that erodes trust in the tool because the damage is easy to miss (2 leading spaces on an
adjacent line) and has nothing to do with the actual edit the user intended.

**Entity Scope:** `None` -- browser widget/UI logic defect, not entity directory instances.

## How Discovered

During task #182's bug-hunting pass across the 9 previously-uninvestigated `module/shader/`
crates, `blockIndent`/`sectionsSplit` were identified as pure, DOM-free string logic reachable
without a browser/WebGPU shim (unlike the rest of `controls.js`), and hand-traced against
selections landing exactly at a line boundary.

## Minimum Reproducible Example

**Verify Command:**
```bash
cd /home/user1/pro/lib/yrd_gamedev/cgtools/module/shader/shader_chunks_preview_web
node --test tests/controls_indent_test.mjs
```
**Expected** (fixed): 4/4 pass. **Actual** (pre-fix): the first 2 tests (single-line and
multi-line whole-line-selection indent) fail -- the line after the selection gains 2 unwanted
leading spaces; e.g. `blockIndent('AAAA\nBBBB\nCCCC\n', 5, 10, false)` produced
`'AAAA\n  BBBB\n  CCCC\n'` (with `CCCC` also indented) instead of `'AAAA\n  BBBB\nCCCC\n'`.

## Root Cause

`blockIndent()` slices the selected block via `value.slice(blockStart, blockEnd)` then
`.split('\n')`s it into "lines" to process one at a time. When the selection ends exactly at a
line boundary (`atLineStart` true), the sliced block itself ends with a trailing `\n`, and
`"...\n".split('\n')` always appends one extra empty-string element that is not a real line --
just the empty span between the last real `\n` and the slice end. The old code treated that
phantom element as a real line: on indent it unconditionally prepended 2 spaces to it, and
because nothing in the reconstructed string separated that phantom edit from
`value.slice(blockEnd)` (the untouched remainder, starting with the next line's first real
character), the 2 spaces landed on the line after the selection instead of doing nothing.

## Why Not Caught

This crate has zero pre-existing automated test coverage (disclosed in its own readme.md), and
`blockIndent` itself was not even `export`ed until this fix -- nothing outside `controls.js`
could reach it to test it in the first place. No manual test session happened to select exactly
at a line boundary and notice the adjacent-line drift.

## Fix Applied (2026-08-18)

`blockIndent()` now strips the trailing `\n` from the sliced block *before* splitting (only when
`atLineStart`), splits only the real line content, then re-appends the stripped `\n` to the
rejoined result -- so `split('\n')` never manufactures a phantom trailing "line" for the
indent/outdent `.map()` to corrupt. The non-`atLineStart` path is untouched: `trailingNewline` is
`''` and `body` is the raw slice unchanged there, so `newBlock` is byte-identical to the pre-fix
computation for every selection that doesn't end at a line boundary. `blockIndent` is also now
`export`ed (previously module-private) specifically so this regression test can reach it
directly.

**New regression tests** (`tests/controls_indent_test.mjs`, new file, Node's built-in test
runner, zero external dependencies): 4 cases -- single-line whole-line selection, multi-line
whole-line selection, whole-line outdent (confirmed already correct pre-fix, kept as a
regression guard), and a non-boundary selection (confirmed unaffected by the fix, no regression).

## Verification

`longrun`-detached where applicable; this crate's test is plain `node --test` (no cargo
involved), run directly (well under any timeout ceiling) from repo root, no `git stash`.

- **Pre-fix (RED):** `node --test tests/controls_indent_test.mjs` against the pre-fix
  `controls.js`: 2 passed, 2 failed (the boundary-selection indent cases).
- **Post-fix (GREEN):** same command: 4 passed, 0 failed -- independently re-run and confirmed by
  the orchestrating session, not only the investigating fork.

## Generalized Version

Any code that slices a text buffer with `value.slice(a, b).split('\n')` and processes every
element as "a line" must account for a trailing separator manufacturing a phantom empty final
element -- `"x\n".split('\n')` is `["x", ""]`, not `["x"]`. The phantom element is falsy-looking
(an empty string) but still very much present and iterated, so it silently slips through any
check that only guards against `undefined`/missing array elements. Applies to any language's
`split`, not just JavaScript's.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-18 | filed + fixed + verified | Found by a fork investigating `shader_chunks_cli_core`/`shader_chunks_preview`/`shader_chunks_preview_web` (task #182, parallel with 2 sibling forks). Fix and regression test written by the fork with a `BUG-XXX` placeholder (forks in this batch were instructed not to self-file, to avoid a 3-way concurrent-write race on the shared bug ledger); this report and its real ID were assigned by the orchestrating session after independently reading the actual committed diff, hand-tracing the phantom-trailing-line mechanism against the fix, and re-running the regression suite directly. |
