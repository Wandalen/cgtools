# BUG-116: `shader_chunks_query_core`'s `format::table` (plain) output truncates at `width::`, contradicting `docs/cli/format/01_table_plain.md`'s documented wrap-onto-continuation-lines contract

- **Severity:** Medium
- **state:** Unverified
- **Affects:** Any `table`-format (plain, the `.list` default) query output from `shader_chunks_query_core::chunks_query` with `width::` set on a result set wide enough to trigger truncation — confirmed concretely for the real `name`/`description`/`tags`/`depends_on` `list` view at `width::30`
- **Component:** `module/shader/shader_chunks_query_core` (`src/lib.rs`'s `render_table` closure, `Table` branch); root mechanism NOT confirmed to live in `data_fmt` 0.8.1 alone — isolated `data_fmt`-only reproduction attempts failed (see `## Evidence Table` E4-E6), so the trigger may involve the real `CHUNKS` dataset's specific content or the query_core call path itself, not yet traced to a specific source line
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **Related Bugs:** [BUG-115](../verified/115_query_markdown_width_truncation_overridden_by_auto_wrap.md) — same `render_table` closure, same `data_fmt` 0.8.1 dependency, same `width::` parameter; discovered as a byproduct of BUG-115's own fix-workflow Step 10 docs sweep, confirmed as a distinct, pre-existing, separate defect via direct pristine-code testing (not caused or fixed by BUG-115's change)

## Symptom

```bash
# Real query_core output, table (plain) format, width::30 (wrong per docs — truncates
# instead of wrapping onto continuation lines)
$ cd /tmp/mre115b && cargo run 2>&1 | tail -6
...
srgb                         Linear-to-sRGB and sRGB-to-...  category:color                  (none)
tonemap_aces                 ACES filmic tone map from H...  category:color                  (none)
gaussian_weight              Unnormalized 1D Gaussian we...  category:filter                 (none)
---
contains ellipsis: true
```

Every long cell (e.g. `description`) is cut to a single line with a trailing `...` — no
continuation lines appear anywhere in the output, contradicting
`docs/cli/format/01_table_plain.md`'s documented structure: "long cells wrap onto continuation
lines... columns left-aligned and padded (capped by `width::`)".

## Impact

**Who is affected:** Anyone running `sch list` (or any direct caller of
`shader_chunks_query_core::chunks_query` with the default `format::table`) with `width::` set on
a result set wide/varied enough to trigger the cap — `format::table` is `.list`'s own default
format, so this is reached by any width-capped default-format query, not an edge case requiring
an explicit `format::table`.

**What breaks:** Silent — no error, no warning. Output renders and is technically readable, but
violates the documented "wrap onto continuation lines" structure (`01_table_plain.md`); cells are
truncated with `...` instead, the same visible shape as `markdown` format's (correct, post-BUG-115)
contract. Anything relying on `table` format's documented multi-line-per-record shape to
distinguish it from `markdown`'s single-line-per-record shape gets the wrong one.

**Magnitude:** Every `table`-format query invocation with `width::` set on a wide-enough result —
confirmed for the real, unfiltered `list` view (4 columns) at `width::30`, representative of
ordinary usage.

**Entity Scope:** None — a code-level formatting defect, not an operational-entity concern.

## How Discovered

Surfaced as a byproduct of BUG-115's own fix-workflow Step 10 (`design/l2_universal.rulebook.md`'s
`dev/arch/l2_gov.rulebook.md § Development Process : Procedure - Bug-Fixing Workflow` PROC2 Step
10 substep 2 — sweep `docs/` for stale restatements of the changed contract). Grepping
`docs/cli/` for `max_column_width`/`auto_wrap` while closing BUG-115 surfaced
`docs/cli/format/01_table_plain.md`'s different (wrap, not truncate) documented contract for the
`table` format. Initial concern was that BUG-115's own fix might have newly broken this contract;
cross-testing against the pristine (pre-BUG-115) `lib.rs` confirmed the mismatch already existed
before any BUG-115 change — a separate, pre-existing defect, not a regression.

```bash
$ git show HEAD:module/shader/shader_chunks_query_core/src/lib.rs > /tmp/-116_pristine_lib.rs
$ diff /tmp/-116_pristine_lib.rs module/shader/shader_chunks_query_core/src/lib.rs | head -5
# (temporarily swapped pristine content into the real file, rebuilt, re-ran
#  /tmp/mre115b, then restored — see History; diff confirms restoration was exact)
$ cd /tmp/mre115b && cargo run 2>&1 | tail -1
contains ellipsis: true
# Pristine (pre-BUG-115) code truncates identically to post-fix code -- confirms
# the table_plain mismatch is independent of BUG-115's own change either way.
```

## Minimum Reproducible Example

```bash
rm -rf /tmp/mre115b && mkdir -p /tmp/mre115b/src
cat > /tmp/mre115b/Cargo.toml <<'EOF'
[package]
name = "mre115b"
version = "0.1.0"
edition = "2021"

[dependencies]
shader_chunks_query_core = { path = "/home/user1/pro/lib/yrd_gamedev/cgtools/module/shader/shader_chunks_query_core" }
EOF
cat > /tmp/mre115b/src/main.rs <<'EOF'
use shader_chunks_query_core::{ QueryParams, chunks_query };

fn main()
{
  let mut params = QueryParams::list_defaults();
  params.width = 30;
  // format left at default (Table/plain)
  let output = chunks_query( &params ).expect( "table query should succeed" );
  println!( "{output}" );
  println!( "contains ellipsis: {}", output.contains( "..." ) );
}
EOF
cd /tmp/mre115b && cargo run 2>&1 | tail -3
```

**Expected** (per `docs/cli/format/01_table_plain.md` — long cells wrap onto continuation lines,
no `...` markers):
```
contains ellipsis: false
```

**Actual**:
```
contains ellipsis: true
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre115b && cargo run 2>&1 | tail -1
# "contains ellipsis: true" = bug present; "false" = fixed
```
**What:** Violates `docs/cli/format/01_table_plain.md`'s documented "long cells wrap onto
continuation lines" structure for `format::table` (plain) — actual output truncates with `...`
instead, the same visible shape `markdown` format uses.

**Known MRE limitation (check 205):** this MRE path-depends on the internal, unpublished
`shader_chunks_query_core` crate rather than being fully self-contained against only the
published `data_fmt` crate (unlike BUG-115's own MRE, which depended on `data_fmt` alone). This
is not a shortcut — three independent attempts at a `data_fmt`-only reduction were made and all
three **failed to reproduce the symptom** (wrapped correctly instead of truncating):

```bash
# All three run via /tmp/mre116, depending only on data_fmt = "0.8.1" (no internal crate):
# 1. Single row, TableConfig::plain().with_max_column_width(30)      -> contains ellipsis: false
# 2. Five rows of varying realistic length, same config              -> contains ellipsis: false
# 3. Thirty rows (five patterns x6, duplicated), same config         -> contains ellipsis: false
```

This rules out both "`TableConfig::plain()` never wraps, regardless of content" and "row
count/scale alone triggers truncation" as explanations (see `## Hypothesis Table` H1, now
Falsified) — the real 50-row `CHUNKS` dataset via `shader_chunks_query_core` remains the only
currently-known reliable reproduction. Whoever picks this up should attempt a data_fmt-only
reduction using the *actual* `CHUNKS` row content (not synthetic approximations) before assuming
the internal crate's call path itself is implicated.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `TableConfig::plain()`'s rendering path never actually achieves multi-line wrap in `data_fmt` 0.8.1, regardless of content — a blanket config-level defect. | ❌ Falsified | Three independent `data_fmt`-only reductions (1 row; 5 rows of realistic varying length; 30 duplicated rows) all wrap correctly (`contains("...")`: false) under the identical `TableConfig::plain().with_max_column_width(30)` config that truncates for the real dataset. `plain()` itself is capable of wrapping — the defect is not in the config alone. | E4, E5, E6 |
| H2 | Row count/scale alone (real dataset has 50 rows vs. the reductions' 1-30) triggers the divergence — e.g. a column-width-computation effect that only manifests past some row-count threshold. | ❌ Falsified | The 30-duplicated-row reduction (E6) — closest in scale to the real 50-row dataset — still wrapped correctly. Scale alone does not explain it. | E6 |
| H3 | Something specific to the real `CHUNKS` dataset's actual content (a specific cell shape, e.g. an unbroken long token that defeats word-wrap, or a specific column combination), or something in `shader_chunks_query_core`'s own call path distinct from a bare `data_fmt` call (`RowBuilder`/`build_view` usage, or another `QueryParams` field), is the actual trigger. | _Investigation ongoing._ | Neither sub-cause distinguished yet — H1 and H2 are both ruled out, narrowing the search space, but the actual mechanism remains unidentified. Whoever picks this up should start with the MRE's own suggested next step (data_fmt-only reduction using the *real* `CHUNKS` row content verbatim, not synthetic approximations) to determine whether it's data-shape-specific or call-path-specific. | E1, E2, E4, E5, E6 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `/tmp/mre115b` run against pristine `lib.rs` (`git show HEAD:module/shader/shader_chunks_query_core/src/lib.rs`, `render_table` with no `auto_wrap` override at all) | `Table` format at `width::30`, real 50-row `CHUNKS` dataset, still truncates (`contains("...")`: true) — confirms the mismatch predates and is independent of BUG-115's own fix. | H3 (real symptom baseline) |
| E2 | BUG-115's own E6 (`/tmp/mre115` wide-table run) | The *same* pristine code, at the *same* `width::30`-equivalent cap, `Markdown` format instead wraps (`contains("...")`: false pre-fix) — establishing that `plain()` and `markdown()` diverge in outcome for the real dataset under the identical `auto_wrap: true` default. | H3 |
| E4 | `/tmp/mre116` run 1 — single row, `data_fmt` 0.8.1 direct (no internal crate), `TableConfig::plain().with_max_column_width(30)` | Wraps correctly (`contains("...")`: false, 6 lines) — `plain()` is not inherently broken. | H1 ❌ |
| E5 | `/tmp/mre116` run 2 — five rows of realistic varying length, same config | Wraps correctly (`contains("...")`: false, 18 lines) — realistic multi-row content alone does not trigger it either. | H1 ❌ |
| E6 | `/tmp/mre116` run 3 — thirty rows (five patterns duplicated x6), same config | Wraps correctly (`contains("...")`: false, 98 lines) — scale approaching the real dataset's 50 rows still does not trigger it. | H1 ❌, H2 ❌ |

## Root Cause

_Investigation ongoing._ Confirmed real, reproducible symptom (E1, E2): the real 50-row `CHUNKS`
dataset, via `shader_chunks_query_core::chunks_query` with `format::table` (default) and
`width::30`, truncates instead of wrapping — independent of BUG-115's own fix either way. Ruled
out (H1, H2 — E4, E5, E6): the defect is neither a blanket `TableConfig::plain()` config-level
break, nor a simple row-count/scale effect — three independent `data_fmt`-only reductions,
including one approaching the real dataset's scale, all wrap correctly under the identical
config. Not yet identified (H3): the actual trigger — either a specific content shape in the
real `CHUNKS` data that these synthetic reductions didn't replicate, or something in
`shader_chunks_query_core`'s own call path (`RowBuilder`/`build_view` usage, or a `QueryParams`
field beyond `width`) distinct from a bare `data_fmt` call. Candidate next step for whoever picks
this up: a `data_fmt`-only reduction using the real `CHUNKS` row content verbatim (not
approximated), to determine whether the trigger is data-shape-specific or call-path-specific
before instrumenting `data_fmt` internals directly.

## Why Not Caught

No test exercises `chunks_query`'s `table` (plain) output against `width::` at a scale that would
distinguish wrap from truncate — `shader_chunks_query_core_test.rs` has no `Table`-format
counterpart to BUG-115's `query_markdown_format_renders_pipe_table_with_heading_and_width`.
`docs/cli/format/01_table_plain.md` was written describing intended/expected behavior without a
regression test enforcing it, allowing documentation and implementation to silently diverge.

## Fix Location

Not yet identified — Root Cause (H3) is not yet at HIGH confidence, so no fix has been attempted.
Once traced, the fix will land in one of: `data_fmt` 0.8.1 itself (if the real content shape
exposes an internal `plain()` wrap defect), or `shader_chunks_query_core/src/lib.rs`'s
`render_table` closure (if a workaround/explicit config override is warranted, mirroring
BUG-115's own `truncate: bool` parameter shape) — H1/H2 being ruled out means a config-only
workaround at the call site is less likely to be sufficient than BUG-115's fix was, since no
config combination tested so far reproduces or resolves the divergence.

## Prevention

Once fixed: add a regression test exercising `chunks_query` with `OutputFormat::Table` against the
full real `shader_chunks_core::CHUNKS` dataset at a `width::` that exceeds the resolved terminal
width, asserting continuation-line wrapping (no `...` markers, output line count > row count) —
mirroring `query_markdown_format_renders_pipe_table_with_heading_and_width`'s own shape but for the
opposite (wrap, not truncate) contract.

**Pitfall:** Two sibling output formats sharing one code path (`render_table`) can have divergent
per-format documented contracts (truncate vs. wrap) that silently drift out of sync with actual
behavior when only one format has regression coverage — audit *all* format branches sharing a
closure/function when fixing a defect in just one of them, not only the one that surfaced it.

## Generalized Version

**Broken assumption:** "`TableConfig::plain()` (the `table` format's config), as used by
`shader_chunks_query_core::chunks_query` against the real `CHUNKS` dataset, wraps long cells onto
continuation lines when capped by `with_max_column_width`, as documented in
`docs/cli/format/01_table_plain.md`" — not confirmed true for the real dataset; empirically,
output truncates with `...` instead, the same visible shape as `markdown()`'s (correct,
documented) truncate contract. **Not** a general property of `TableConfig::plain()` itself —
confirmed capable of correct wrapping under matched config, realistic content, and matched scale
in isolation (H1, H2 both falsified; see `## Evidence Table` E4-E6).

Fails when:
1. `with_max_column_width` is set, AND
2. Queried through `shader_chunks_query_core::chunks_query` against the real `CHUNKS` dataset
   specifically — the precise trigger (real content shape vs. call-path difference) is the open
   question in `## Root Cause` (H3), not yet reduced to a general, portable rule.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Filed as a disclosed follow-up from BUG-115's own Step 10 docs sweep; symptom and pre-existing (non-regression) status confirmed via direct pristine-code MRE testing. |
| 2026-08-15 | corrected | Initial Hypothesis Table claimed H1 (blanket `TableConfig::plain()` config defect) as ✅ Verified from symptom evidence alone. Three follow-up `data_fmt`-only reduction attempts (E4-E6: 1 row, 5 rows, 30 rows) all wrapped correctly under the identical config, falsifying H1 and a follow-up row-count hypothesis (H2). Root Cause narrowed to H3 (real-dataset-specific or call-path-specific trigger, still open) rather than left overstated. |

## Refs: src/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_query_core/src/lib.rs` | None yet — `render_table`'s `Table` branch (`chunks_render`, `OutputFormat::Table => render_table( TableConfig::plain(), false )`) is the eventual fix site once Root Cause (H2) reaches HIGH confidence. |

## Refs: tests/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_query_core/tests/shader_chunks_query_core_test.rs` | None yet — a `Table`-format regression test (mirroring `query_markdown_format_renders_pipe_table_with_heading_and_width`) should be added alongside the eventual fix. |
