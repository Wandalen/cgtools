# BUG-115: `shader_chunks_query_core`'s documented `width::` truncation is silently overridden by `data_fmt`'s auto-wrap once a query's total row width exceeds 120 columns

- **Severity:** Medium
- **state:** Completed
- **Affects:** Any `markdown`/`table`-format query output from `shader_chunks_query_core::chunks_query` wide enough (columns × capped-width sum) to exceed `data_fmt`'s resolved terminal width — confirmed concretely for the real `name`/`description`/`tags`/`depends_on` `list` view at `width::30`
- **Component:** `module/shader/shader_chunks_query_core` (`src/lib.rs`'s `render_table` closure); root mechanism lives in the external `data_fmt` 0.8.1 dependency
- **repo_identity:** self
- **Filed:** 2026-08-15
- **filed_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/bug/
- **verified_by:** user1@w002/home/user1/pro/lib/yrd_gamedev/cgtools/task/verified/
- **verification_date:** 2026-08-16
- **Related Bugs:** [BUG-116](../completed/116_query_table_plain_width_wraps_documentation_mismatch.md) — originally shared the `render_table` closure (since split by BUG-116's own fix into independent `Table`/`Markdown` arms of `chunks_render`), same `data_fmt` 0.8.1 dependency, same `width::` parameter; discovered as a byproduct of this bug's own Step 10 docs sweep, confirmed as a distinct, pre-existing, separate defect (not caused or fixed by this bug's change)

## Symptom

```bash
# Real query_core output, markdown format, width::30 (wrong — wraps instead of truncating)
$ cargo nextest run -p shader_chunks_query_core query_markdown_format_renders_pipe_table_with_heading_and_width
thread '...' panicked at module/shader/shader_chunks_query_core/tests/shader_chunks_query_core_test.rs:324:3:
width::30 must truncate long descriptions:
─── Chunks ──────────────────────────────────────────────────────────────────
| name                        | description                 | tags                  | depends_on                |
|-----------------------------|-----------------------------|-----------------------|---------------------------|
| hash21                      | Single-value hash of a 2D   | category:hash         | (none)                    |
|                             | point into [0, 1).          |                       |                           |
| value_noise                 | Bilinear-interpolated value | category:noise        | hash21                    |
|                             | noise sampled at a 2D       |                       |                           |
|                             | point, in [0, 1).           |                       |                           |
# ... 47 more rows, same shape — zero "..." markers anywhere in the entire output

# Same underlying truncation mechanism on a narrow table (correct — /tmp/mre115, 1 col, width::20)
| Description           |
|------------------------|
| This is a very lo... |
# total row width (~24) stays under data_fmt's resolved terminal width (120 fallback); auto_wrap
# never engages here, so with_max_column_width's single-line truncate path runs as documented
```

## Impact

**Who is affected:** Anyone running `sch list`/`sch query`-family commands (or any direct caller of
`shader_chunks_query_core::chunks_query`) with `format::markdown` or `format::table` and a `width::`
narrower than the natural content, against a result set wide/varied enough to push the table's total
row width over the resolved terminal width (120 columns by default in this workspace).

**What breaks:** Silent — no error, no warning. Output still renders and is technically readable, but
every cell that should have been cut to `width::N` with a trailing `...` is instead word-wrapped across
multiple lines, violating `docs/cli/param/21_width.md`'s documented "cells longer than N chars truncate
with `...`" contract. Anything piping this output expecting single-line-per-row records (e.g. `wc -l`,
`grep`, downstream parsers) gets multi-line rows instead.

**Magnitude:** Every `markdown`/`table` query invocation whose result set is wide enough — confirmed for
the real, unfiltered `list` view (4 columns: name/description/tags/depends_on) at `width::30`, which is
representative of ordinary usage, not an edge case.

**Entity Scope:** None — a code-level formatting defect, not an operational-entity concern.

## How Discovered

Surfaced as a pre-existing failing test discovered incidentally while completing an unrelated feature
(making every bundled shader chunk previewable) in this same session — flagged rather than silently
fixed or dropped, then pursued as its own bug-fixing workflow per explicit user authorization.

```bash
$ longrun .launch dir::/home/user1/pro/lib/yrd_gamedev/cgtools/module -- env CARGO_TARGET_DIR="$SP/target" cargo nextest run -p shader_chunks_query_core query_markdown_format_renders_pipe_table_with_heading_and_width
    FAIL [   0.022s] (1/1) shader_chunks_query_core::shader_chunks_query_core_test
    query_markdown_format_renders_pipe_table_with_heading_and_width
$ longrun .wait
# exit: 1 — test result: FAILED. 0 passed; 1 failed
```

## Minimum Reproducible Example

```bash
mkdir -p /tmp/mre115/src
cat > /tmp/mre115/Cargo.toml <<'EOF'
[package]
name = "mre115"
version = "0.1.0"
edition = "2021"

[dependencies]
data_fmt = "0.8.1"
EOF
cat > /tmp/mre115/src/main.rs <<'EOF'
use data_fmt::{ RowBuilder, TableFormatter, TableConfig, Format };

fn main()
{
  // Every column's content at/over the 30-char cap, mirroring a real
  // multi-column query view -- capped total (4*30 + 3 sep + 8 padding +
  // 2 border = 133) exceeds data_fmt's resolved terminal width (hardcoded
  // 120 fallback absent `terminal_size`/`$COLUMNS`).
  let wide = RowBuilder::new( vec![ "name".into(), "description".into(), "tags".into(), "depends_on".into() ] )
    .add_row( vec![
      "d2_sdf_equilateral_triangle".into(),
      "Signed distance from a 2D point to an equilateral triangle of the given circumradius, apex up.".into(),
      "category:sdf, dim:2d, technique:analytic".into(),
      "d2_sdf_box, d2_sdf_round_box".into(),
    ] )
    .build_view();
  let out = Format::format( &TableFormatter::with_config( TableConfig::markdown().with_max_column_width( Some( 30 ) ) ), &wide ).unwrap();
  println!( "{out}" );
  assert!( out.contains( "..." ), "with_max_column_width(30) must truncate, not wrap" );
}
EOF
cd /tmp/mre115 && cargo run 2>&1 | tail -20
```

**Expected** (per `docs/cli/param/21_width.md` — single line per row, `...` markers present):
```
| name                          | description                    | tags                           | depends_on                    |
|--------------------------------|---------------------------------|---------------------------------|--------------------------------|
| d2_sdf_equilateral_triangl... | Signed distance from a 2D p... | category:sdf, dim:2d, techn... | d2_sdf_box, d2_sdf_round_b... |
```

**Actual**:
```
| name                        | description                | tags                          | depends_on        |
|-----------------------------|-----------------------------|--------------------------------|--------------------|
| d2_sdf_equilateral_triangle | Signed distance from a 2D  | category:sdf, dim:2d,          | d2_sdf_box,        |
|                             | point to an equilateral    | technique:analytic             | d2_sdf_round_box   |
|                             | triangle of the given      |                                |                    |
|                             | circumradius, apex up.     |                                |                    |

thread 'main' panicked at src/main.rs:19:3:
with_max_column_width(30) must truncate, not wrap
```

**Verify Command** (≤3 lines, standalone):
```bash
cd /tmp/mre115 && cargo run 2>&1 | grep -c '\.\.\.'
# 0 = bug present (no truncation markers); >0 = fixed
```
**What:** Violates `docs/cli/param/21_width.md`'s documented contract that `with_max_column_width`
guarantees single-line truncation — `data_fmt`'s independent `auto_wrap` default silently overrides it
once the table's total (already-capped) row width exceeds the resolved terminal width.

## Hypothesis Table

| ID | Hypothesis | State | Summary | Evidence |
|----|-----------|--------|---------|----------|
| H1 | `auto_wrap` defaulting to `true` in `TableConfig::markdown()`/`plain()` unconditionally causes `with_max_column_width` to be ignored, regardless of table width. | ❌ Disproved | `table_config.rs:126` sets `auto_wrap: true` by `Default`; `markdown()`/`plain()` (`table_config.rs:165-198`) never disable it. | E2 |
| H2 | `should_auto_wrap` only overrides truncation when the SUM of already-capped column widths (+ separators/padding/borders) exceeds `resolve_terminal_width()`'s resolved value — below that sum, `with_max_column_width` is honored via ordinary single-line truncation. | ✅ Root Cause | `auto_fit.rs:88-97`'s `should_auto_wrap` computes `total > terminal`; `mod.rs:402-410` caps each column at `max_column_width` BEFORE that sum is computed. | E1, E3, E4, E5, E6 |
| H3 | `resolve_terminal_width()` deterministically resolves to the hardcoded `120` fallback in this workspace (not a flaky/environment-dependent real-TTY width), because `data_fmt`'s `terminal_size` feature is not compiled in and `$COLUMNS` is unset in the test/CI environment. | ✅ Verified | `auto_fit.rs:14-39`'s 3-tier resolution; `Cargo.lock`'s locked `data_fmt` deps omit `terminal_size`. | E7, E8 |
| H4 | Once `auto_wrap` applies, `apply_auto_wrap` pre-fits every wrapped line to ≤ the column's (capped) width, so the later `truncate_cell` call in the rendering path is a structural no-op — explaining why zero `...` markers ever appear rather than a mix of truncated and wrapped cells. | ✅ Verified | `rendering.rs:65,176`'s two `truncate_cell` call sites; the multi-line path's cells already fit, making truncation vacuous. | E9 |

## Evidence Table

| # | Location | What it shows | Hypothesis |
|---|----------|---------------|------------|
| E1 | `data_fmt-0.8.1/src/config/table_config.rs:126` | `Default::default()` sets `auto_wrap: true`; confirmed `markdown()` (191-198) and `plain()` (165-168) never call `.with_auto_wrap(false)`. | H2 ✅ |
| E2 | `/tmp/mre115` narrow-table run (Terminal output) | 1-column table, `max_column_width(20)`, `markdown()` truncates correctly (`contains("...")`: true) — total row width ≈24, well under 120. If H1 (unconditional override) were correct, this case would also wrap; it doesn't. | H1 ❌ |
| E3 | `data_fmt-0.8.1/src/formatters/table/auto_fit.rs:88-97` | `should_auto_wrap` returns `compute_total_row_width(column_widths) > resolve_terminal_width()`. | H2 ✅ |
| E4 | `data_fmt-0.8.1/src/formatters/table/mod.rs:402-410` | `calculate_column_widths_for_rows` caps every column at `max_column_width` BEFORE returning — the array `should_auto_wrap` sums is already post-cap, not natural content width. | H2 ✅ |
| E5 | `data_fmt-0.8.1/src/formatters/table/auto_fit.rs:63-85` | `compute_total_row_width` = `sum(column_widths) + separator_width*(n-1) + cell_inner_padding*2*n + border(2)`. | H2 ✅ |
| E6 | `/tmp/mre115` wide-table run (Terminal output) | 4-column table, every column's content at/over `max_column_width(30)`, `markdown()` wraps instead of truncating (`contains("...")`: false; panics on the documented-truncation assertion). Capped total = 4×30 + 3 + 8 + 2 = 133 > 120. | H2 ✅ |
| E7 | `data_fmt-0.8.1/src/formatters/table/auto_fit.rs:14-39` | `resolve_terminal_width`: explicit override → `$COLUMNS` → feature-gated `terminal_size` → hardcoded `120`. | H3 ✅ |
| E8 | `cgtools/Cargo.lock` (`data_fmt` dependency block) | Locked `data_fmt` 0.8.1 dependency list omits `terminal_size` — the feature is not compiled in for this workspace. | H3 ✅ |
| E9 | `data_fmt-0.8.1/src/formatters/table/rendering.rs:65,176` | Two `truncate_cell` call sites; the multi-line/wrapped path's lines are already ≤ column width, so truncation there is a no-op. | H4 ✅ |

## Root Cause

```
render_table() sets with_max_column_width(Some(30))       (shader_chunks_query_core/src/lib.rs:566)
                     but never disables auto_wrap
TableConfig::markdown()/plain() leave auto_wrap: true      (data_fmt table_config.rs:126,165-198)
calculate_column_widths_for_rows() caps each column
  at 30, THEN returns the capped widths                    (data_fmt mod.rs:402-410)
should_auto_wrap( capped_widths )
  = sum(capped_widths) + overhead > 120 (hardcoded)         (data_fmt auto_fit.rs:63-97)
  = true for a 4-column table near/at the cap  ✗
apply_auto_wrap() word-wraps every cell to fit
  instead of taking the single-line truncate_cell path      (data_fmt mod.rs:199-207)
```

`with_max_column_width` and `auto_wrap` are two independent `data_fmt` mechanisms with no coordination:
the former is meant to guarantee single-line truncation, but the latter — left at its type default
(`true`) by both `TableConfig::plain()` and `::markdown()` — silently pre-empts it whenever the *sum* of
the already-capped column widths crosses the resolved terminal width, which in this workspace is always
the hardcoded `120` fallback since `terminal_size` isn't compiled in. `shader_chunks_query_core` requests
truncation via `with_max_column_width` but never disables `auto_wrap`, so any output wide enough — the
real 4-column `name`/`description`/`tags`/`depends_on` view is a textbook case — silently wraps instead,
violating the documented (`docs/cli/param/21_width.md`) truncate contract with no error or warning.

## Why Not Caught

No test exercised `chunks_query`'s markdown/table output against the full, real `shader_chunks_core`
dataset with `width::` set until `query_markdown_format_renders_pipe_table_with_heading_and_width` was
added — and that test began failing immediately, rather than being backed by a passing baseline. No
narrower unit test covers `data_fmt`'s interaction between `with_max_column_width` and `auto_wrap` at
this crate's boundary; the only prior width-adjacent coverage used synthetic tables narrow enough to
never cross the 120-column auto-wrap threshold (confirmed by this bug's own `/tmp/mre115` narrow case,
which passes even without the fix).

## Fix Location

`module/shader/shader_chunks_query_core/src/lib.rs:557-582` (`render_table` closure gains a
`truncate: bool` parameter, `true` only for the `Markdown` call site — `Table`/plain's own
documented contract is wrap, not truncate, and must not have `auto_wrap` touched; see
`## Prevention` and BUG-116):

```rust
// Before:
let render_table = | config : TableConfig | -> Result< String, QueryError >
{
  let mut config = config;
  if !params.heading.is_empty()
  {
    config = config.with_heading( Heading::new( params.heading.clone() ) );
  }
  if params.width > 0
  {
    config = config.with_max_column_width( Some( params.width ) );
  }
  Format::format( &TableFormatter::with_config( config ), &view )
  .map_err( | e | QueryError::Render( e.to_string() ) )
};

match params.format
{
  OutputFormat::Table => render_table( TableConfig::plain() ),
  OutputFormat::Markdown => render_table( TableConfig::markdown() ),
  // ...
}

// After:
let render_table = | config : TableConfig, truncate : bool | -> Result< String, QueryError >
{
  let mut config = config;
  if !params.heading.is_empty()
  {
    config = config.with_heading( Heading::new( params.heading.clone() ) );
  }
  if params.width > 0
  {
    config = config.with_max_column_width( Some( params.width ) );
    if truncate
    {
      // Fix(BUG-115): with_max_column_width alone doesn't guarantee truncation
      // Root cause: data_fmt's auto_wrap (default true) silently wraps instead of truncating once total capped row width exceeds the resolved terminal width (120 fallback) — markdown's documented contract (docs/cli/param/21_width.md) is truncate, so auto_wrap must be disabled here; table_plain's documented contract (docs/cli/format/01_table_plain.md) is wrap-onto-continuation-lines, so it must keep auto_wrap at its default (a separate, pre-existing data_fmt gap keeps table_plain from actually achieving that wrap today — tracked as BUG-116, out of this fix's scope)
      // Pitfall: a formatting library's independent config knobs can silently interact — always check whether disabling one (auto_wrap) is correct for every call site sharing the code path, not just the one that surfaced the bug
      config = config.with_auto_wrap( false );
    }
  }
  Format::format( &TableFormatter::with_config( config ), &view )
  .map_err( | e | QueryError::Render( e.to_string() ) )
};

match params.format
{
  OutputFormat::Table => render_table( TableConfig::plain(), false ),
  OutputFormat::Markdown => render_table( TableConfig::markdown(), true ),
  // ...
}
```

**Correction (2026-08-15, caught during Step 10's docs sweep):** the fix originally applied
`.with_auto_wrap(false)` unconditionally whenever `params.width > 0`, for both `Table` and
`Markdown`. Empirically verified against a `/tmp` MRE that this made `Table`-format output also
truncate — but `docs/cli/format/01_table_plain.md` documents `table_plain` as wrapping onto
continuation lines, a *different* contract from markdown's truncate. Cross-checking against the
pristine (pre-BUG-115) code showed `Table` format was *already* truncating instead of wrapping
before any fix was applied — so the blanket version was not a regression in practice, but it was
still the wrong fix shape (silently relying on a coincidence instead of encoding the actual
per-format contract). Narrowed to `truncate: bool`, `true` only for `Markdown`. The pre-existing
`table_plain` wrap/truncate mismatch itself is real, confirmed, and out of this bug's scope —
filed separately as BUG-116.

## Prevention

Add a regression test exercising `chunks_query` with `OutputFormat::Markdown`/`Table` against the full
real `shader_chunks_core::CHUNKS` dataset (not a narrow synthetic table) at a `width::` that would trip
`auto_wrap` pre-fix — `query_markdown_format_renders_pipe_table_with_heading_and_width` already does
this; it only needs the fix applied to pass. Detection command:
```bash
grep -n 'with_max_column_width' module/shader/*/src/lib.rs
```
— **not** every hit should be paired with `.with_auto_wrap( false )`: whether to disable auto-wrap
depends on that call site's own documented per-format contract (markdown here: truncate, so disable;
`table_plain`: wrap, so leave the default — see the Fix Location correction note and BUG-116). Manually
confirm each hit's pairing (or deliberate non-pairing) still matches its own format's documented
contract in `docs/cli/`, rather than assuming a uniform rule across every call site.

**Pitfall:** When a formatting library exposes two independent knobs that can both reshape output (here:
truncate-via-cap vs. wrap-via-fit), setting one without checking the other's default leaves a silent,
condition-dependent behavior switch — always audit sibling config knobs for interaction, not just the
one you're directly setting.

## Generalized Version

**Broken assumption:** "Setting `with_max_column_width(Some(n))` on a `data_fmt` `TableConfig` guarantees
single-line truncation" — only true when the table's total row width (summed across all already-capped
columns, plus separators/padding/borders) stays at or under the resolved terminal width (`$COLUMNS`, or a
hardcoded `120` fallback absent the `terminal_size` feature).

Fails for any `data_fmt` `TableConfig` (`plain()`, `markdown()`, or `Default`) when:
1. `with_max_column_width` is set without also calling `.with_auto_wrap(false)`, AND
2. The table has enough columns, and/or wide-enough content in enough of them, that the sum of their
   (capped) widths plus rendering overhead exceeds the resolved terminal width, AND
3. The caller never overrides `term_width` explicitly and the environment leaves `$COLUMNS` unset (or the
   `terminal_size` feature is uncompiled, as in this workspace).

**Detection invariant:**
```
∀ TableConfig c with c.max_col_width().is_some():
  c.is_auto_wrap() == false
```

## History

| Date | Event | Notes |
|------|-------|-------|
| 2026-08-15 | filed | Filed during this repo's formal Bug-Fixing Workflow; root cause fully confirmed via `/tmp/mre115` (narrow-passes / wide-fails contrast) before filing. |
| 2026-08-15 | fix_applied | `.with_auto_wrap( false )` added, scoped to `Markdown` only via new `truncate: bool` param — `module/shader/shader_chunks_query_core/src/lib.rs:557-582`. Corrected same-day from an initial blanket (both-format) version after Step 10's docs sweep surfaced `table_plain`'s different wrap contract. |
| 2026-08-15 | verified | `query_markdown_format_renders_pipe_table_with_heading_and_width` passes (`cargo nextest run -p shader_chunks_query_core`, exit 0); full workspace suite green (`verb/test`: 1756/1756 native tests, doctests, clippy, 52/52 wasm32 checks, 3/3 wasm32 test crates — all 0 failed). |
| 2026-08-15 | linked | Filed [BUG-116](../unverified/116_query_table_plain_width_wraps_documentation_mismatch.md) as a disclosed follow-up for `table_plain`'s separate, pre-existing wrap/truncate documentation mismatch surfaced by this bug's own Step 10 docs sweep; `**Related Bugs:**` cross-link added to this file's header. |
| 2026-08-15 | corrected | BUG-116's fix split the previously-shared `render_table` closure into independent `Table`/`Markdown` arms of `chunks_render` — this file's `**Related Bugs:**` line updated from present-tense "same closure" to reflect the split; `## Fix Location`/`## Refs: src/`/code snippets below are left unchanged as accurate history of this bug's own fix shape at the time. |
| 2026-08-16 | completed | Acceptance verification by a distinct session (filer/fixer/self-verifier 2026-08-15, this verifier 2026-08-16). Independently re-read `chunks_render`'s `Markdown` arm (confirmed `config = config.with_auto_wrap( false );` genuinely present, gated on `params.width > 0`, 3-field Fix/Root cause/Pitfall comment intact) and the `bug_reproducer` test (`query_markdown_format_renders_pipe_table_with_heading_and_width` — non-tautological, asserts `output.contains("...")` for a genuinely over-width description). Confirmed the `Table` arm (BUG-116) does not also disable `auto_wrap` (correctly left at its default, bypassed instead via pre-wrap) — no scope overlap between the two fixes. Fresh `cargo nextest run -p shader_chunks_query_core -p shader_chunks_query --all-features` via `longrun`: 32/32 passed. `cargo clippy` (both crates, all-features/all-targets, `-D warnings`): clean. Corrected the stale `**Related Bugs:**` cross-reference (`../unverified/116_...` → `../completed/116_...`, doubly stale: 116 had already moved past `unverified/` before this pass). MAAV Tier 2 Dual-Role Self-Check (`governance/maav.rulebook.md`), covering BUG-115/116/117 together. State → Completed. |

## Verification Record

**Gate Check** · Tier: 2 · Type: Full · Verdict: PASS · Agents: 0 (self, dual-role) · 8/8

| Gate | Name | Prev | Now | Issues | Fixes |
|------|------|------|-----|--------|-------|
| D1 | Completeness | 🟢 | 🟢 | — | — |
| D2 | MRE Validity & Reproducibility | 🟠 | 🟢 | Adversarial pass: `--offline` in the script and Verify Command assumes a pre-cached registry — fails on a genuinely fresh machine per check 205's portability requirement | Dropped `--offline` from both commands; re-ran fresh — exit 101, panic message matches Actual block, Verify Command returns `0` |
| D3 | Cross-Reference Integrity | 🟠 | 🟢 | Adversarial pass: check 306 backreferences missing from both `Refs:`-listed files; E1's Hypothesis cell cited `H1 ✅` even though H1 is ❌ Disproved (E1 only supports the shared precondition, not H1's disproved claim); E1/E7 used a "(symptom)" qualifier on code-inspection evidence, which FI033 reserves for timing/observation evidence | Added backreference comments to `lib.rs:566` and the test file; dropped E1's H1 citation and narrowed H1's Evidence cell to `E2` (the actual disproof); removed the "(symptom)" qualifier from E1 and E7 |
| D4 | Root Cause Quality | 🟡 | 🟢 | Adversarial pass: Fix Location cited `561-567`, but 561 is actually the sibling `with_heading` block's opening brace — the `width` block is `564-567`, shifted to `564-568` once the backreference comment was inserted | Corrected the cited range to `564-568`, matching current file content exactly |
| D5 | Execution Scope | 🟢 | 🟢 | Adversarial pass considered whether this is really a `data_fmt` (upstream) bug rather than a `cgtools` one — the chosen fix is fully self-contained in this repo and resolves the documented contract without any upstream change | — |
| D6 | Crate Scope Unity | 🟢 | 🟢 | — | — |
| D7 | Crate Locality | 🟢 | 🟢 | Adversarial pass considered whether a CLI-layer crate should own the fix instead — `render_table`/`TableConfig` construction lives entirely in `query_core`'s own `lib.rs`, confirmed the leaf owner | — |
| D8 | Crate Single Responsibility | 🟢 | 🟢 | Adversarial pass considered whether disabling `auto_wrap` adds a new "work around data_fmt quirks" responsibility — it doesn't; configuring `TableConfig` knobs has always been part of this closure's existing job | — |

**Reproduced:** YES — exit 101, 2026-08-15

## Refs: src/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_query_core/src/lib.rs` | Add `.with_auto_wrap( false )` alongside `with_max_column_width` in `render_table`, scoped to the `Markdown` call site only via a new `truncate: bool` parameter (lines 557-582). |

## Refs: tests/

| File | Change |
|------|--------|
| `module/shader/shader_chunks_query_core/tests/shader_chunks_query_core_test.rs` | Mark the existing failing test `bug_reproducer(BUG-115)` once the fix lands. |
