# BUG-117: `shader_chunks_query`'s `docs/cli/param/21_width.md` still claimed `width::` truncates in `table` format, contradicting BUG-116's fix (`table` now wraps)

- **Severity:** Low
- **state:** Completed
- **Affects:** `module/shader/shader_chunks_query/docs/cli/param/21_width.md` (Purpose, Example,
  Notes) and its test-spec mirror `module/shader/shader_chunks_query/tests/docs/cli/param/21_width.md`
  (EC-1) — documentation only, no source code path affected
- **Component:** `module/shader/shader_chunks_query` (docs/cli/param/21_width.md,
  tests/docs/cli/param/21_width.md) — distinct from `shader_chunks_query_core`, which owns the
  actual rendering code BUG-115/BUG-116 fixed
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** [BUG-115](../completed/115_query_markdown_width_truncation_overridden_by_auto_wrap.md),
  [BUG-116](../completed/116_query_table_plain_width_wraps_documentation_mismatch.md) — same
  `width::`/`data_fmt` subject area; BUG-116's fix (manual `WrapFormatter` pre-wrap for
  `format::table`) made `table` output wrap instead of truncate, but this doc file was never
  updated to reflect that split — a documentation-only aftershock of BUG-116's own change,
  discovered by a workspace-wide docs-vs-code sweep run under the same standing bug-hunt mandate

## Symptom

`docs/cli/param/21_width.md` described `width::` as truncating cells in *both* `table` and
`markdown` output:

- Purpose: "Caps every column's width in `table` and `markdown` output, truncating longer cells."
- Example: `list width::12  # cells longer than 12 chars truncate with '...'` — `list`'s own
  default format is `table` (plain), so this example command's own comment was wrong.
- Notes: "Truncation is `data_fmt`'s `with_max_column_width` behavior — an ellipsis marks cut
  cells."

Its test-spec mirror, `tests/docs/cli/param/21_width.md` EC-1, restated the same claim
("`width::N` truncates over-long cells with `...` in table/markdown") backed only by a
markdown-format test — it did not cite either of BUG-116's own two regression tests, which prove
the opposite for `table`.

Since BUG-116's fix, `format::table` (plain) — `.list`'s default — wraps long cells onto
continuation lines instead of truncating them (confirmed by
`query_table_format_wraps_short_name_long_description_row_instead_of_truncating` and
`query_table_format_full_dataset_never_truncates_at_width` in
`shader_chunks_query_core_test.rs`, both currently passing). Only `markdown` format still
truncates. `21_width.md` was never updated when BUG-116 split `table` and `markdown` onto
independent code paths with genuinely different `width::` contracts — it was the one doc file
left describing the pre-BUG-116, single-shared-path behavior.

## Impact

**Who is affected:** Anyone reading `docs/cli/param/21_width.md` (or its test-spec mirror) to
learn what `width::` does under `format::table` — the default format for `.list` — got a
factually wrong description and a copy-pasteable example command whose own documented comment was
false.

**What breaks:** Nothing at runtime — this was a pure documentation defect, not a code path. The
risk was purely to anyone trusting the doc over the actual (correct) behavior: filing a false bug
report against `table` format for "not truncating," writing calling code that assumes truncation
(e.g. computing expected output line counts), or a future contributor "fixing" `table` format back
to truncate to match this stale doc, silently reintroducing BUG-116.

**Magnitude:** One doc file plus its test-spec mirror; the underlying code (already fixed by
BUG-115/BUG-116) was unaffected and already correctly tested.

**Entity Scope:** None — a documentation-level defect, not an operational-entity concern.

## How Discovered

Surfaced by a workspace-wide docs-vs-code consistency sweep (read-only `Explore` agent), run under
the standing "continue finding and fixing bugs" mandate as a direct follow-on to BUG-115/BUG-116 —
the sweep specifically looked for other `docs/` files making width/truncate/wrap claims left stale
by BUG-116's format-specific split. Cross-checked directly against `docs/cli/format/01_table_plain.md`
(correct: "long cells wrap onto continuation lines... capped by `width::`") and
`docs/cli/format/04_markdown.md` (correct: "cell width capped by `width::` with `...` truncation")
— both accurate; `21_width.md` alone was the outlier, confirmed via
`git log --oneline -- module/shader/shader_chunks_query/docs/cli/param/21_width.md` showing its
only commit predates both BUG-115's and BUG-116's fixes.

## Minimum Reproducible Example

Not applicable in the usual code-reproduction sense — this was a documentation-only defect with no
executable symptom. The "reproduction" is a direct text/behavior comparison:

```bash
$ grep -n "truncat" module/shader/shader_chunks_query/docs/cli/param/21_width.md
# (pre-fix) matched the blanket "table and markdown... truncating" claim and the
# table-format example's "truncate with `...`" comment

$ cargo test -p shader_chunks_query_core --test shader_chunks_query_core_test \
    query_table_format_wraps_short_name_long_description_row_instead_of_truncating 2>&1 | tail -3
test query_table_format_wraps_short_name_long_description_row_instead_of_truncating ... ok
```

The doc claimed truncation for `table`; the actual, currently-passing test proves wrap. No code
change was needed or made — the defect was entirely in the prose.

**Verify Command** (≤3 lines, standalone):
```bash
grep -c "Caps every column's width in \`table\` and \`markdown\`.*truncating longer cells" \
  module/shader/shader_chunks_query/docs/cli/param/21_width.md
# 0 = fixed (blanket table+markdown truncation claim removed); >0 = bug present
```

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `21_width.md` was never updated when BUG-116 split `table`'s `width::` contract from `markdown`'s — a straightforward doc-staleness aftershock, not a new/independent code-vs-doc mismatch. | ✅ Verified (HIGH confidence) | Direct text comparison against the already-correct sibling docs (`01_table_plain.md`, `04_markdown.md`) plus `git log` confirming `21_width.md`'s only commit predates both BUG-115 and BUG-116. No alternative hypothesis was entertained — the mechanism is self-evident from the diff between what the doc said and what the (already-fixed, already-tested) code does. | E1, E2, E3 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `docs/cli/param/21_width.md` (pre-fix): Purpose, Example, Notes | Blanket "`table` and `markdown`... truncating" claim, and a `table`-format example (`list width::12`) commented as truncating. | H1 |
| E2 | `docs/cli/format/01_table_plain.md` line 8, `04_markdown.md` line 8 | Sibling docs correctly and consistently describe the post-BUG-116 split: `table` wraps, `markdown` truncates. | H1 |
| E3 | `shader_chunks_query_core_test.rs::query_table_format_wraps_short_name_long_description_row_instead_of_truncating` and `::query_table_format_full_dataset_never_truncates_at_width` (both currently passing, added by BUG-116) | Direct, currently-passing proof that `table` format wraps, not truncates — contradicted `21_width.md`'s pre-fix claim for the actual, current, tested behavior. | H1 |

## Root Cause

**HIGH confidence.** `docs/cli/param/21_width.md` (and its test-spec mirror) was written when
`chunks_render`'s `Table` and `Markdown` output shared a single closure (`render_table`) with one
common `width::` contract (truncate). BUG-116 split that closure into independent match arms with
genuinely different `width::` behavior per format (`table` now pre-wraps via `WrapFormatter`;
`markdown` still truncates via `with_auto_wrap(false)`) — but only updated the two format-specific
docs (`01_table_plain.md`, `04_markdown.md`) that directly described the changed rendering
structure. `21_width.md` describes the *parameter*, not a specific format's rendering structure,
so it fell outside the sweep scope BUG-116's own fix workflow covered and was left stating the
pre-split, no-longer-accurate blanket claim.

## Why Not Caught

BUG-116's own Step 10 docs sweep (per `design/l2_universal.rulebook.md`'s Bug-Fixing Workflow)
grepped `docs/cli/` for `max_column_width`/`auto_wrap` — literal library symbol names — which
`21_width.md` didn't mention (it describes the parameter in end-user terms, "truncating longer
cells," not by library API name), so that grep didn't surface it. No test asserts doc-text content
against actual behavior (docs are prose, not code under test), so nothing short of a dedicated
docs-vs-code sweep would catch this class of drift.

## Fix Location

`module/shader/shader_chunks_query/docs/cli/param/21_width.md` (Purpose, Example, Notes sections)
and `module/shader/shader_chunks_query/tests/docs/cli/param/21_width.md` (EC-1, Simple
Co-Dependencies, Test Coverage Summary) — rewritten to state the format-specific contract (`table`
wraps onto continuation lines, `markdown` truncates with `...`) matching
`01_table_plain.md`/`04_markdown.md` and the current, tested code. No source code changed — this
bug has no `Refs: src/` section.

## Prevention

No new test added — the two existing BUG-116 regression tests
(`query_table_format_wraps_short_name_long_description_row_instead_of_truncating`,
`query_table_format_full_dataset_never_truncates_at_width`) already are, and remain, the
executable proof of the behavior this doc now correctly describes; the test-spec mirror's EC-1 was
split into EC-1a (table, cites both `table`-format tests by name) and EC-1b (markdown, cites the
existing markdown test), so a future reader lands on current, format-specific evidence instead of
one test standing in for both formats.

**Pitfall:** A fix workflow's own "sweep docs/ for stale claims" step, if scoped by grepping for
the *library's* internal symbol names, can miss doc files that describe the same behavior in
end-user vocabulary instead. When a fix changes a shared code path's behavior per-branch, sweep
*every* doc file that documents the affected parameter or format — not only the ones whose prose
happens to name-check the library API just changed.

## Generalized Version

**Broken assumption:** "Updating the format-specific docs (`01_table_plain.md`, `04_markdown.md`)
that directly describe a changed rendering structure is sufficient to keep all documentation
consistent." False when a *parameter*-level doc (`21_width.md`) independently restates the same
behavioral claim in its own words — parameter docs and format docs can describe the same contract
from two different angles, and only updating one angle leaves the other stale.

**Confirmed general rule:** When a fix changes a parameter's behavior conditionally on some other
dimension (here: format), every doc file that documents that parameter — not just the docs for the
dimension that now varies — must be checked for restatements of the old, now-format-independent
claim.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Found by a workspace-wide docs-vs-code consistency sweep (read-only Explore agent) run under the standing bug-hunt mandate; root cause immediately clear (doc staleness aftershock of BUG-116's own fix), filed with H1 already Verified rather than left open. |
| 2026-08-15 | fixed | `docs/cli/param/21_width.md` (Purpose/Example/Notes) and its test-spec mirror (EC-1 split into EC-1a/EC-1b) rewritten to state the format-specific `width::` contract (`table` wraps, `markdown` truncates), matching `01_table_plain.md`/`04_markdown.md` and the current, tested code. No source changed. |
| 2026-08-15 | verified | Re-read corrected doc text against `01_table_plain.md`/`04_markdown.md` and the two live BUG-116 regression tests — consistent; grep-confirmed no remaining blanket-truncation claim. Tier 2 Dual-Role Self-Check run (`## Verification Record`). |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read the full current `docs/cli/param/21_width.md` (confirmed the format-specific contract genuinely present: "`table` wraps longer cells onto continuation lines; `markdown` truncates them with `...`" — no stale blanket-truncation claim remains anywhere in Purpose/Example/Notes) and the full test-spec mirror `tests/docs/cli/param/21_width.md` (confirmed EC-1a/EC-1b split genuinely present, citing real, currently-passing test functions: EC-1a → `query_table_format_wraps_short_name_long_description_row_instead_of_truncating` + `query_table_format_full_dataset_never_truncates_at_width`; EC-1b → `query_markdown_format_renders_pipe_table_with_heading_and_width`). Cross-checked both against the now-closed BUG-115/BUG-116 code fixes — consistent. Corrected both stale `**Related Bugs:**` cross-references (`../verified/115_...`/`../verified/116_...` → `../completed/115_...`/`../completed/116_...`). MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-115/116/117 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟠 | 🟢 | Confirming pass drafted all 12 FI008 sections plus Verification Record, judged complete. Adversarial pass checked `task/bug/readme.md`'s Open Bugs table directly (`grep -c "BUG-117"`) rather than assuming the row existed because BUG-115/116's did — found 0 matches: the row was genuinely missing. | Added the BUG-117 row to `task/bug/readme.md`'s Open Bugs table, matching BUG-115/116's exact column format. |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Adversarial pass re-ran the file's own Verify Command grep against the real post-fix file (not asserted from memory of what the edit should have produced): `grep -c "Caps every column's width in..."` → `0`, confirmed by actual command output. | — |
| D3 | Cross-Reference Integrity | 🟢 | 🟢 | Adversarial pass confirmed both `../format/01_table_plain.md` and `../format/04_markdown.md` relative links from `docs/cli/param/21_width.md` resolve to real files (both read directly earlier this session); confirmed BUG-115/BUG-116's own files need no reciprocal update — their claims about `render_table`'s split are unaffected by this doc-only fix. | — |
| D4 | Root Cause Quality | 🟢 | 🟢 | Adversarial pass did not take the discovering agent's `git log` claim on trust (Stale Evidence Trust risk) — independently re-ran `git log --oneline -- .../21_width.md .../tests/docs/cli/param/21_width.md` directly: single commit `f3fde26a` (the original CLI-crate-split refactor), predating both BUG-115 and BUG-116 — claim holds under independent re-check. | — |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass re-examined the decision to exclude the discovering agent's LOW-confidence secondary note (`01_table_plain.md`/`04_markdown.md` "Rendering mechanism" fields not mentioning `WrapFormatter`) — confirmed exclusion is correct: doesn't misstate observable behavior (agent's own assessment), stays within this bug's one falsifiable claim, avoids scope creep into a non-actionable field. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Adversarial pass ran `git status --porcelain` directly rather than assuming scope from memory — confirmed BUG-117's own changes touch exactly `shader_chunks_query`'s two doc files plus `task/bug/readme.md` and this bug file; other modified paths present in the working tree (`shader_chunks_query_core/src/lib.rs` and its test, `115_...md`, the `116_...md` rename, `114_...md`) predate this bug or belong to unrelated in-flight work, not BUG-117. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Adversarial pass confirmed the fix stayed inside the two files that actually carried the stale claim — no edits to unrelated `docs/cli/param/` files that don't mention truncation/wrap. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Adversarial pass considered whether this bug file itself should have been split (one for the doc fix, one for the test-spec-mirror fix) — no: both are the same root cause, same fix session, same "keep `21_width.md`'s two representations in sync" responsibility; splitting would be bureaucratic overkill for a 2-file, same-cause change. | — |

**Reproduced:** N/A — documentation-only defect, no executable failure state; symptom was the doc
text itself (quoted verbatim above), confirmed stale via direct comparison against already-correct
sibling docs and already-passing tests, 2026-08-15.

## Refs: docs/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_query/docs/cli/param/21_width.md` | Purpose/Example/Notes rewritten from a blanket "table and markdown both truncate" claim to the accurate format-specific contract (table wraps via `WrapFormatter`, markdown truncates via `with_auto_wrap(false)`); added cross-references to `01_table_plain.md`/`04_markdown.md`. |
| `module/shader/shader_chunks_query/tests/docs/cli/param/21_width.md` | EC-1 split into EC-1a (table wraps, cites both BUG-116 regression tests) and EC-1b (markdown truncates, cites the existing markdown test); Simple Co-Dependencies and Test Coverage Summary updated to match (3→4 edge cases). |
