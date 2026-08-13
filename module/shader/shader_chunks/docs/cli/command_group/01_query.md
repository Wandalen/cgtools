# Command Group :: 1. Query

### Pattern

Set-shaped questions over the compiled-in chunk registry
(`shader_chunks_core::CHUNKS`): select rows, filter them, project columns,
and render the result in a chosen output format. `.list` and `.get` are
literally the same engine — one shared routine
(`query_chunks`/`query_routine` in `src/lib.rs`/`src/cli.rs`) behind both
commands, with an identical 20-parameter surface differing only in
defaults and in whether `names` is required. `.tags` is the tag-axis pivot
of the same metadata.

### Purpose

Let a shader author answer "which chunks match, and what do their fields
hold" — from a broad overview (`list` with no arguments) through arbitrary
filter/projection/format combinations, down to the full detail record of
one named chunk (`get`).

### Semantic Coherence Test

"All 3 commands answer a set-shaped metadata question about the compiled-in
chunk registry." `.list` answers "which chunks match these filters, showing
these fields"; `.get` answers the same question with the candidate set
fixed to named chunks and detail-leaning defaults; `.tags` answers "what
tags exist and on which chunks." Every member fits the single sentence.

### Why NOT Split `list` and `get` Into Two Groups

A plausible split is "browse" (`list`, `tags`) vs. "inspect" (`get`).
Rejected: `.get` is not a separate detail machine — it dispatches to the
same `query_routine` as `.list`, accepts the same 19 named parameters plus
`names`, and differs only in defaults (`fields::` gains `stage`+`exports`,
`format::` starts at `expanded`, `names` is required). Splitting them would
draw a group boundary through one function. The genuine boundaries in this
CLI are output-species ones: relationship rendering (→
[Graph](02_graph.md)) and WGSL text production (→
[Compose](03_compose.md)).

### Invariants

- Idempotent: identical input always produces identical output.
- No side effects outside stdout content and process exit code.
- Only `shader_chunks_core::CHUNKS` is consulted — no filesystem,
  environment, or network access.
- `list` and `get` produce byte-identical output under identical explicit
  parameters — defaults are the only behavioral difference.
- Every parameter validation failure exits non-zero with a loud message —
  never a panic, never a silent fallback.

### Referenced Commands

| # | Command | Relationship |
|---|---------|---------------|
| 1 | [`.list`](../command/01_list.md) | Member — query, every chunk by default |
| 2 | [`.get`](../command/02_get.md) | Member — same engine, named chunks, detail defaults |
| 3 | [`.tags`](../command/03_tags.md) | Member — tag-axis pivot of the registry |

**Membership:** these 3 of the 6 commands; the partition across all groups
is stated in [`readme.md`](readme.md).

### Referenced Parameter Groups

| # | Parameter Group | Relationship |
|---|-----------------|--------------|
| 1 | [`filtering`](../param_group/01_filtering.md) | Row selection for `.list`/`.get` |
| 2 | [`projection`](../param_group/02_projection.md) | Column selection for `.list`/`.get` |
| 3 | [`formatting`](../param_group/03_formatting.md) | Output shaping for `.list`/`.get` |

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/command_group/01_query.md`](../../../tests/docs/cli/command_group/01_query.md) | Group-level test specification |
| [`../../../tests/shader_chunks_test.rs`](../../../tests/shader_chunks_test.rs) | `query_list_and_get_defaults_share_engine_and_agree_under_equal_params` proves the shared engine |
| [`../../../tests/cli_subprocess_test.rs`](../../../tests/cli_subprocess_test.rs) | `list_and_get_agree_under_identical_explicit_parameters` proves it end-to-end |

### Typical Patterns

Start broad, then narrow: `list` for the overview; add `pattern::`/`tag::`/
`stage::` filters and a `format::` to taste; `get <name>` when one chunk's
full detail (including `exports` and, via `fields::source`, the WGSL body)
is the goal.

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
