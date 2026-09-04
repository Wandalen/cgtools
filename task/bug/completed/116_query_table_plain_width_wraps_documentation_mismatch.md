# BUG-116: `shader_chunks_query_core`'s `format::table` (plain) output truncates at `width::`, contradicting `docs/cli/format/01_table_plain.md`'s documented wrap-onto-continuation-lines contract

- **Severity:** Medium
- **state:** Completed
- **Affects:** Any `table`-format (plain, the `.list` default) query output from `shader_chunks_query_core::chunks_query` with `width::` set on a row whose capped total row width stays under the resolved terminal width (`120` fallback) despite an individual cell exceeding `width::` — confirmed concretely for the real `name`/`description`/`tags`/`depends_on` `list` view at `width::30`, both for a single short-`name` row (`hash21`) and the full 50-row default dataset
- **Component:** `module/shader/shader_chunks_query_core` (`src/lib.rs`'s `chunks_render` function, `Table` branch); root mechanism confirmed HIGH confidence to live entirely in `data_fmt` 0.8.1's `should_auto_wrap` gate (`auto_fit.rs:88-97`) — see `## Root Cause`
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** [BUG-115](../completed/115_query_markdown_width_truncation_overridden_by_auto_wrap.md) — originally shared the `render_table` closure (since split: `chunks_render`'s `Table` and `Markdown` arms are now independent, see `## Fix Location`), same `data_fmt` 0.8.1 dependency, same `width::` parameter; discovered as a byproduct of BUG-115's own fix-workflow Step 10 docs sweep, confirmed as a distinct, pre-existing, separate defect via direct pristine-code testing (not caused or fixed by BUG-115's change)

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
| H3 | Something specific to the real `CHUNKS` dataset's actual content (a specific cell shape, e.g. an unbroken long token that defeats word-wrap, or a specific column combination), or something in `shader_chunks_query_core`'s own call path distinct from a bare `data_fmt` call (`RowBuilder`/`build_view` usage, or another `QueryParams` field), is the actual trigger. | ✅ Verified (refined into H4) | Data-shape-specific, not call-path-specific: a `data_fmt`-only reduction using the real `CHUNKS` row content (not synthetic) reproduced the bug once the `name` field was long enough, isolating the trigger to `name` field length specifically. Refined below into H4's precise, source-confirmed mechanism. | E7, E8 |
| H4 | `data_fmt`'s `should_auto_wrap` gates wrapping on whether the CAPPED TOTAL ROW WIDTH (sum of every column's already-capped width, plus separators/padding/border) exceeds the resolved terminal width (`120` fallback) — not on whether any individual cell's content exceeds `max_column_width`. A short `name` column keeps the row's total width under `120` even when `description` alone exceeds `width::`, so `auto_wrap` never fires and `truncate_cell` truncates instead. | ✅ Verified (HIGH confidence) | Source-read confirmation of `auto_fit.rs:88-97`'s exact condition (`total > terminal`), cross-checked against a clean, reproducible threshold: varying only `name`'s length (1-10 synthetic chars vs. the original 28-char value) against otherwise-identical rows — name ≤10 chars truncates, name=28 chars wraps, with the crossover matching the arithmetic (`compute_total_row_width`, `auto_fit.rs:63-85`) exactly. | E7, E8, E9 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `/tmp/mre115b` run against pristine `lib.rs` (`git show HEAD:module/shader/shader_chunks_query_core/src/lib.rs`, `render_table` with no `auto_wrap` override at all) | `Table` format at `width::30`, real 50-row `CHUNKS` dataset, still truncates (`contains("...")`: true) — confirms the mismatch predates and is independent of BUG-115's own fix. | H3 (real symptom baseline) |
| E2 | BUG-115's own E6 (`/tmp/mre115` wide-table run) | The *same* pristine code, at the *same* `width::30`-equivalent cap, `Markdown` format instead wraps (`contains("...")`: false pre-fix) — establishing that `plain()` and `markdown()` diverge in outcome for the real dataset under the identical `auto_wrap: true` default. | H3 |
| E4 | `/tmp/mre116` run 1 — single row, `data_fmt` 0.8.1 direct (no internal crate), `TableConfig::plain().with_max_column_width(30)` | Wraps correctly (`contains("...")`: false, 6 lines) — `plain()` is not inherently broken. | H1 ❌ |
| E5 | `/tmp/mre116` run 2 — five rows of realistic varying length, same config | Wraps correctly (`contains("...")`: false, 18 lines) — realistic multi-row content alone does not trigger it either. | H1 ❌ |
| E6 | `/tmp/mre116` run 3 — thirty rows (five patterns duplicated x6), same config | Wraps correctly (`contains("...")`: false, 98 lines) — scale approaching the real dataset's 50 rows still does not trigger it. | H1 ❌, H2 ❌ |
| E7 | `/tmp/mre116` run 4-5 — full real 50-row dataset (parsed from an actual `chunks_query` JSON dump), then binary-search bisection over row prefixes | Full dataset reproduces (`contains_ellipsis: true`); bisection finds the smallest triggering prefix is length 1 — row `hash21` alone reproduces the bug in isolation. Proves the trigger is data-shape-specific (a property of certain rows), not scale-specific. | H3 ✅, H1 ❌ (reconfirmed) |
| E8 | `/tmp/mre116` run 6-7 — field-isolation test on `hash21`'s real content, then systematic `name`-length sweep (1, 5, 10, 28 synthetic chars) holding `description`/`tags`/`depends_on` constant | Trigger tracks `name` field length, not `description` length: `name` ≤10 chars → truncates; `name` = 28 chars (matching a real long-name chunk) → wraps. Clean, reproducible threshold. | H3 ✅, H4 ✅ |
| E9 | Direct read of `data_fmt-0.8.1/src/formatters/table/auto_fit.rs:88-97` (`should_auto_wrap`) and `:63-85` (`compute_total_row_width`) | `should_auto_wrap` returns `compute_total_row_width(column_widths) > resolve_terminal_width()` — an aggregate-row-width condition, not a per-cell one. `compute_total_row_width` sums every column's already-capped width plus separator/padding/border overhead. A short `name` keeps this sum under the `120` fallback even when `description` alone exceeds `width::`, exactly matching E7/E8's empirical threshold. Source-level confirmation, HIGH confidence. | H4 ✅ |
| E10 | Post-fix verification (`/tmp/verify116`, and `shader_chunks_query_core_test.rs`'s 2 new `bug_reproducer(BUG-116)` tests) | `hash21` alone at `width::30` now wraps (`Single-value hash of a 2D` / `point into [0, 1).` on separate lines, `contains_ellipsis: false`); full default dataset at `width::30` has zero `...` occurrences; `fullscreen_triangle` (already-correct long-name case) still wraps, no regression. Narrow suite: 32/32 passed (`cargo nextest run -p shader_chunks_query_core`), clippy clean. | Fix confirmed |

## Root Cause

**HIGH confidence, source-confirmed (H4).** `data_fmt` 0.8.1's `should_auto_wrap`
(`formatters/table/auto_fit.rs:88-97`) gates pre-wrapping on whether the **capped total row
width** exceeds the resolved terminal width (`120` hardcoded fallback, since `terminal_size` isn't
compiled into this workspace) — not on whether any individual cell's content exceeds
`max_column_width`:

```rust
pub( super ) fn should_auto_wrap( &self, column_widths : &[ usize ] ) -> bool
{
  if !self.config.is_auto_wrap() { return false; }
  if !self.config.col_widths_override().is_empty() { return false; }
  if column_widths.is_empty() { return false; }
  if self.config.is_csv_or_tsv() { return false; }
  let total = self.compute_total_row_width( column_widths );
  let terminal = self.resolve_terminal_width();
  total > terminal
}
```

(The three intermediate guards — explicit column-width override, empty columns, CSV/TSV output — don't apply to `chunks_render`'s call shape: no override is set, `column_widths` is never empty for a real query, and `table`/`markdown` aren't CSV/TSV. The decisive condition for this bug is the final `total > terminal`.)

`compute_total_row_width` (`auto_fit.rs:63-85`) sums every column's *already-capped* width plus
separator/padding/border overhead. For the real `CHUNKS` dataset, a short `name` column (e.g.
`hash21`, 6 chars) keeps this sum under `120` even when `description` alone exceeds `width::30` —
so `auto_wrap` never fires, and rendering falls through to `truncate_cell` (`ansi_str.rs:164`,
called from `rendering.rs:65`/`:176`), which `...`-truncates any cell still exceeding its capped
column width regardless of the wrap decision. Longer-`name` chunks (e.g. a 28-char name) push the
row's total width over `120` and DO trigger `auto_wrap` correctly — this is exactly why the bug is
data-shape-dependent rather than a blanket `plain()` defect (H1, ruled out) or a scale effect (H2,
ruled out): the real dataset mixes both short- and long-`name` rows, so some rows wrap correctly
by coincidence while others silently truncate.

`chunks_render`'s `Table` branch (pre-fix) relied entirely on `auto_wrap`'s default (`true`) to
achieve `table_plain`'s documented wrap contract, with no independent wrap mechanism of its own —
so it inherited this threshold gate's blind spot unconditionally.

## Why Not Caught

No test exercises `chunks_query`'s `table` (plain) output against `width::` at a scale that would
distinguish wrap from truncate — `shader_chunks_query_core_test.rs` has no `Table`-format
counterpart to BUG-115's `query_markdown_format_renders_pipe_table_with_heading_and_width`.
`docs/cli/format/01_table_plain.md` was written describing intended/expected behavior without a
regression test enforcing it, allowing documentation and implementation to silently diverge.

## Fix Location

`module/shader/shader_chunks_query_core/src/lib.rs`, `chunks_render` function, `OutputFormat::Table`
match arm. Rather than patching a `data_fmt` config knob (no combination tested resolves the
divergence — H1/H2 ruled this out), the fix bypasses `should_auto_wrap`'s terminal-width gate
entirely for this branch: every cell's text is pre-wrapped directly via
`WrapFormatter::with_config( WrapConfig::new().width( params.width ) ).wrap_joined( &cell.text )`
— the same primitive `auto_wrap` uses internally (`apply_auto_wrap`, `auto_fit.rs:179-227`) — before
building the `TableView`, so wrapping no longer depends on the row's aggregate width at all. The
`Table` arm was split out of the `render_table` closure it previously shared with `Markdown` (which
keeps BUG-115's `with_auto_wrap( false )` fix unchanged, against the original un-wrapped `view`).

## Prevention

Added: `query_table_format_wraps_short_name_long_description_row_instead_of_truncating` and
`query_table_format_full_dataset_never_truncates_at_width`
(`shader_chunks_query_core_test.rs`) — the first exercises exactly the short-`name`/long-
`description` shape that exposes the aggregate-width gate (`hash21` alone, `width::30`), the
second sweeps the full real dataset for any remaining `...`. Mirrors
`query_markdown_format_renders_pipe_table_with_heading_and_width`'s shape but for the opposite
(wrap, not truncate) contract.

**Pitfall:** Two sibling output formats sharing one code path (`render_table`) can have divergent
per-format documented contracts (truncate vs. wrap) that silently drift out of sync with actual
behavior when only one format has regression coverage — audit *all* format branches sharing a
closure/function when fixing a defect in just one of them, not only the one that surfaced it. And
when a library's "auto" behavior is gated on an *aggregate* condition (total row width) rather than
a *per-cell* one, single-cell test fixtures (BUG-115's own coverage used a single wide cell/short
row) can systematically miss the gate's blind spot — vary the shape (which column is long, how the
others compare) not just the content length.

## Generalized Version

**Broken assumption:** "`TableConfig::plain()`'s `auto_wrap` (default `true`) wraps any cell whose
content exceeds `with_max_column_width`, as documented in `docs/cli/format/01_table_plain.md`" —
false in general. `auto_wrap` only fires when the row's **capped total width** (all columns
summed, plus overhead) exceeds the resolved terminal width (`120` fallback) — a per-row aggregate
condition, not a per-cell one.

**Confirmed general rule** (H4, source- and empirically-verified): for any `TableConfig::plain()`/
`::markdown()` row with `auto_wrap` at its default, an over-`max_column_width` cell wraps only if
its row's *other* columns are collectively wide enough to push the total over `120` — otherwise it
silently truncates via `truncate_cell`, regardless of how far that one cell's content exceeds the
cap. This is a property of `data_fmt` 0.8.1 itself (confirmed via direct source read of
`auto_fit.rs`), not specific to `shader_chunks_query_core`'s call path or the `CHUNKS` dataset —
any `data_fmt` consumer relying on `auto_wrap`'s default for a documented wrap contract is exposed
to the same gate. The fix applied here (manual `WrapFormatter` pre-wrap, bypassing the gate) is the
general-purpose workaround for any caller wanting unconditional per-cell wrap regardless of row
shape.

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Filed as a disclosed follow-up from BUG-115's own Step 10 docs sweep; symptom and pre-existing (non-regression) status confirmed via direct pristine-code MRE testing. |
| 2026-08-15 | corrected | Initial Hypothesis Table claimed H1 (blanket `TableConfig::plain()` config defect) as ✅ Verified from symptom evidence alone. Three follow-up `data_fmt`-only reduction attempts (E4-E6: 1 row, 5 rows, 30 rows) all wrapped correctly under the identical config, falsifying H1 and a follow-up row-count hypothesis (H2). Root Cause narrowed to H3 (real-dataset-specific or call-path-specific trigger, still open) rather than left overstated. |
| 2026-08-15 | root-caused | Full real-dataset MRE (E7) bisected the 50-row failure to a single triggering row (`hash21`); a systematic `name`-length sweep (E8) found a clean threshold, confirming H3 (data-shape-, not call-path-specific). Direct source read of `data_fmt-0.8.1/src/formatters/table/auto_fit.rs` (E9) confirmed the exact mechanism at HIGH confidence: `should_auto_wrap` gates on capped total row width vs. resolved terminal width (`120` fallback), not per-cell content — refined into H4, ✅ Verified. |
| 2026-08-15 | fixed | `chunks_render`'s `Table` arm split out of the `render_table` closure it shared with `Markdown`; now manually pre-wraps every cell via `WrapFormatter`/`WrapConfig` (the same primitive `auto_wrap` uses internally) before building the view, bypassing `should_auto_wrap`'s gate entirely. `Markdown`'s BUG-115 fix left unchanged. |
| 2026-08-15 | verified | Added 2 regression tests (`query_table_format_wraps_short_name_long_description_row_instead_of_truncating`, `query_table_format_full_dataset_never_truncates_at_width`) plus a 3-field source comment. Narrow suite: 32/32 passed (`cargo nextest run -p shader_chunks_query_core --all-features`); `cargo clippy -p shader_chunks_query_core --all-targets --all-features -- -D warnings` clean. Post-fix empirical spot-check (E10) confirms `hash21` alone, the full dataset, and the already-correct `fullscreen_triangle` long-name case all now wrap with zero `...` occurrences. |
| 2026-08-15 | verified | Full workspace suite green (`verb/test` via `longrun`, elapsed 1180s): nextest 1762/1762, doctests 54/54 crates ok, clippy clean, wasm32 check 52/52 examples, wasm32 test 3/3 crates — 0 failures anywhere, 0 stray `error`/`panicked` lines across the full log. Tier 2 Dual-Role Self-Check (`## Verification Record`, D1-D8, 8/8) run against this file itself; adversarial pass caught and fixed 2 real defects (D3: stale cross-file `render_table` reference in both this file and BUG-115's; D4: Root Cause code excerpt had silently dropped 3 guard clauses vs. the real `should_auto_wrap`). |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read `chunks_render`'s `Table` arm (confirmed the manual `WrapFormatter::with_config( WrapConfig::new().width( params.width ) ).wrap_joined( &cell.text )` pre-wrap genuinely present, applied before `build_view`, gated on `params.width > 0`, split cleanly from the `Markdown` arm's independent `with_auto_wrap( false )` fix) and both `bug_reproducer(BUG-116)` tests (`query_table_format_wraps_short_name_long_description_row_instead_of_truncating` — non-tautological, asserts specific wrapped-content lines plus `!output.contains("...")` plus `output.lines().count() > 3` for the `hash21` short-name/long-description row; `query_table_format_full_dataset_never_truncates_at_width` — sweeps the full 50-row dataset for zero `...` occurrences). Confirmed no scope overlap with BUG-115 (the `Markdown` arm does not use `WrapFormatter`; the `Table` arm does not call `with_auto_wrap`). Fresh `cargo nextest run -p shader_chunks_query_core -p shader_chunks_query --all-features` via `longrun`: 32/32 passed. `cargo clippy` (both crates, all-features/all-targets, `-D warnings`): clean. Corrected the stale `**Related Bugs:**` cross-reference (`../verified/115_...` → `../completed/115_...`). MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-115/116/117 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | All 12 FI008 sections + both `Refs:` present — confirmed by direct re-read of the full file. | — |
| D2 | MRE Validity & Reproducibility | 🟢 | 🟢 | Adversarial pass re-ran the file's own Verify Command verbatim (`cd /tmp/mre115b && cargo run 2>&1 \| tail -1`) rather than trusting the earlier `/tmp/verify116` spot-check — confirms `contains ellipsis: false` against the actual committed fix, not just a throwaway harness. | — |
| D3 | Cross-Reference Integrity | 🟠 | 🟢 | Adversarial pass found this file's own `**Related Bugs:**` line, and BUG-115's reciprocal line, both still asserted present-tense "same `render_table` closure" — stale, since this bug's fix split that closure into independent `Table`/`Markdown` arms. | Corrected both files' `**Related Bugs:**` lines to past-tense "originally shared... since split"; added a `corrected` History entry to BUG-115's file. |
| D4 | Root Cause Quality | 🟡 | 🟢 | Adversarial pass diffed the Root Cause section's `should_auto_wrap` excerpt against a fresh read of `auto_fit.rs:88-97` — the excerpt had silently dropped 3 guard clauses (`col_widths_override`, empty-columns, CSV/TSV) present in the real function; cited line range (88-97) was correct, only the pasted body was incomplete. | Replaced with the verbatim function body (all 4 guards + `pub( super )`) and added a parenthetical confirming the 3 extra guards don't apply to `chunks_render`'s call shape. |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass considered whether `should_auto_wrap`'s per-cell blind spot warrants an upstream `data_fmt` issue instead of a local workaround — the manual-`WrapFormatter` fix is fully self-contained and consistent with BUG-115's own local-fix precedent for the same dependency. | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | Adversarial pass confirmed only `shader_chunks_query_core/src/lib.rs`, its own `tests/`, and the two bug-tracking `.md` files were touched — no cross-crate scope creep. | — |
| D7 | Crate Locality | 🟢 | 🟢 | Adversarial pass considered whether `Table`/`Markdown` should share a wrap-handling helper below `chunks_render` — their wrap behavior is now genuinely divergent (pre-wrap vs. truncate), so keeping them as separate match arms (still sharing `with_heading`/`build_view`) is correct, not accidental duplication. | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Adversarial pass considered whether manually invoking `WrapFormatter` adds a new "text-wrapping" responsibility to `chunks_render` — it doesn't; `WrapFormatter`/`WrapConfig` are the same `data_fmt` library `chunks_render` already fully depends on for rendering, just a different piece of its public API. | — |

**Reproduced:** YES — `contains ellipsis: true` (pre-fix, via `/tmp/mre115b`), 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_query_core/src/lib.rs` | `chunks_render`: split `OutputFormat::Table` into its own match arm (previously shared `render_table` closure with `Markdown`); `Table` arm now manually pre-wraps every cell via `WrapFormatter::with_config( WrapConfig::new().width( params.width ) ).wrap_joined( &cell.text )` before building the view, bypassing `data_fmt`'s `should_auto_wrap` gate. Added `DecoratedText`, `WrapConfig`, `WrapFormatter` to the `use data_fmt::{ ... }` import. `Fix(BUG-116)`/`Root cause`/`Pitfall` 3-field comment added at the fix site. `Markdown` arm unchanged (BUG-115's `with_auto_wrap( false )` fix retained). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_query_core/tests/shader_chunks_query_core_test.rs` | Added `query_table_format_wraps_short_name_long_description_row_instead_of_truncating` (single-row `hash21` reproducer, `bug_reproducer(BUG-116)` with 5-section doc comment) and `query_table_format_full_dataset_never_truncates_at_width` (full-dataset sweep regression guard). |
