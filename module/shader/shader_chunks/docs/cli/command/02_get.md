# Command :: 2. get

### Description

Queries *named* chunks with the same engine and parameters as
[`list`](01_list.md) — detail columns and expanded records by default.
The two commands share one routine (`query_chunks` behind
`query_routine`) and one 20-parameter surface; `get` differs only in its
defaults (`fields::` gains `stage`+`exports`, `format::` starts at
`expanded`) and in requiring at least one chunk name. Use it once `list`
has told you which chunk name(s) to inspect.

-- **Parameters:** names (required, ≥1), plus the 19 shared named query
   parameters — [filtering](../param_group/01_filtering.md),
   [projection](../param_group/02_projection.md),
   [formatting](../param_group/03_formatting.md)
-- **Exit Codes:** 0 (success) | 1 (missing `names`; validation — unknown
   chunk/field, invalid enum or negative integer value) | 2 (render
   error — internal, `data_fmt` formatting failed)
-- **Modes:** `count::1` (aggregate count instead of records);
   `format::names` (bare name lines for shell pipelines)

### Syntax
```bash
shader_chunks get <names...> [param::value ...]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `names` | [`ChunkName`](../param/02_names.md) (list, positional) | — | Yes (≥1) | Which bundled chunks to show, in the given order (duplicates allowed) |
| `pattern::` | [String](../param/03_pattern.md) | off | No | Substring filter on chunk names |
| `case::` | [`Switch`](../param/04_case.md) | `false` | No | Case-sensitive `pattern::`/`exports::` matching |
| `tag::` | [`TagSelector`](../param/05_tag.md) (list) | off | No | Tag selectors: `group:tag` pair or bare `tag` |
| `tags_mode::` | [`TagsMode`](../param/06_tags_mode.md) | `any` | No | Combine `tag::` selectors: union or intersection |
| `stage::` | [`StageSelector`](../param/07_stage.md) | `any` | No | Stage filter: `any` \| `none` \| literal |
| `depends_on::` | [`ChunkName`](../param/08_depends_on.md) | off | No | Keep only chunks depending on this chunk |
| `transitive::` | [`Switch`](../param/09_transitive.md) | `false` | No | Widen `depends_on::` to the transitive closure |
| `exports::` | [String](../param/10_exports.md) | off | No | Substring filter over export signatures |
| `roots::` | [`Switch`](../param/11_roots.md) | `false` | No | Keep only chunks nothing else depends on |
| `leaves::` | [`Switch`](../param/12_leaves.md) | `false` | No | Keep only chunks with no dependencies |
| `fields::` | [`FieldName`](../param/13_fields.md) (list) | `name,description,stage,tags,depends_on,exports` | No | Fields to project, in order |
| `count::` | [`Switch`](../param/14_count.md) | `false` | No | Print only the matched-chunk count |
| `format::` | [`OutputFormat`](../param/15_format.md) | `expanded` | No | `table` \| `markdown` \| `expanded` \| `json` \| `yaml` \| `names` |
| `sort::` | [`SortKey`](../param/16_sort.md) | `input` | No | `input` \| `name` \| `stage` \| `description` |
| `order::` | [`SortOrder`](../param/17_order.md) | `asc` | No | `asc` \| `desc` |
| `limit::` | [`NonNegativeInteger`](../param/18_limit.md) | `0` (unlimited) | No | Keep at most N chunks |
| `offset::` | [`NonNegativeInteger`](../param/19_offset.md) | `0` | No | Skip the first N chunks |
| `heading::` | [String](../param/20_heading.md) | off | No | Heading line (table/markdown only) |
| `width::` | [`NonNegativeInteger`](../param/21_width.md) | `0` (auto) | No | Max column width (table/markdown only) |

### Examples
```bash
shader_chunks get hash21
# -[ RECORD 1 ]
# name        | hash21
# description | Single-value hash of a 2D point into [0, 1).
# stage       | (none)
# tags        | category:hash
# depends_on  | (none)
# exports     | fn hash21(p: vec2f) -> f32

shader_chunks get hash21 fbm3 fields::name,source format::yaml
# - name: hash21
#   source: |-
#     ... raw WGSL body ...
# - name: fbm3
#   ...

shader_chunks get bogus_chunk
# unknown chunk: `bogus_chunk` (see `list` for valid names)
# (exit 1)

shader_chunks get
# Argument Error: The required argument 'names' is missing
# (exit 1)
```

### Notes
- `list <names...>` and `get <names...>` produce byte-identical output
  under identical explicit parameters — the defaults are the only
  behavioral difference (verified by
  `cli_subprocess_test.rs::list_and_get_agree_under_identical_explicit_parameters`).
- Filters still apply to the named selection — `get hash21 fbm3
  tag::fractal` keeps only `fbm3`.
- `stage`/`depends_on`/`exports` render `(none)` when absent — cells are
  never blank.
- Output format: [`expanded`](../format/05_expanded.md) by default; all 6
  query formats selectable via `format::`.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.list`](01_list.md) | Same engine, names optional, overview defaults |
| 2 | [`.tree`](04_tree.md) | See a chunk's dependency chain instead of its flat detail |
| 3 | [`.compose`](05_compose.md) | Combine chunks into composed WGSL |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*

---

**Category:** chunk
**Complexity:** 8
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low
