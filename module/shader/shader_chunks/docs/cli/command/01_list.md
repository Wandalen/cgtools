# Command :: 1. list

### Description

Queries the bundled shader chunks: filter, sort, project, and format —
every chunk by default. `list` and [`get`](02_get.md) run the *same*
engine (`chunks_query` behind `query_routine`) with the same 20-parameter
surface; `list`'s defaults lean overview (all chunks, 4 columns, plain
table), `get`'s lean detail. Use it to discover chunks by name fragment,
tag, stage, dependency relationship, or export signature.

-- **Parameters:** names (optional), plus the 19 shared named query
   parameters — [filtering](../param_group/01_filtering.md),
   [projection](../param_group/02_projection.md),
   [formatting](../param_group/03_formatting.md)
-- **Exit Codes:** 0 (success, including an empty match) | 1 (validation —
   unknown chunk/field, invalid enum or negative integer value) |
   2 (render error — internal, `data_fmt` formatting failed)
-- **Modes:** `count::1` (aggregate count instead of rows);
   `format::names` (bare name lines for shell pipelines)

### Syntax
```bash
shader_chunks list [names...] [param::value ...]
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `names` | [`ChunkName`](../param/02_names.md) (list, positional) | every chunk | No | Selection: which chunks enter the query, in which order |
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
| `fields::` | [`FieldName`](../param/13_fields.md) (list) | `name,description,tags,depends_on` | No | Columns to project, in order |
| `count::` | [`Switch`](../param/14_count.md) | `false` | No | Print only the matched-chunk count |
| `format::` | [`OutputFormat`](../param/15_format.md) | `table` | No | `table` \| `markdown` \| `expanded` \| `json` \| `yaml` \| `names` |
| `sort::` | [`SortKey`](../param/16_sort.md) | `input` | No | `input` \| `name` \| `stage` \| `description` |
| `order::` | [`SortOrder`](../param/17_order.md) | `asc` | No | `asc` \| `desc` |
| `limit::` | [`NonNegativeInteger`](../param/18_limit.md) | `0` (unlimited) | No | Keep at most N chunks |
| `offset::` | [`NonNegativeInteger`](../param/19_offset.md) | `0` | No | Skip the first N chunks |
| `heading::` | [String](../param/20_heading.md) | off | No | Heading line (table/markdown only) |
| `width::` | [`NonNegativeInteger`](../param/21_width.md) | `0` (auto) | No | Max column width (table/markdown only) |

### Examples
```bash
shader_chunks list
# name                 description                        tags           depends_on
# -------------------  ---------------------------------  -------------  -----------
# hash21               Single-value hash of a 2D point    category:hash  (none)
#                      into [0, 1).
# ...

shader_chunks list pattern::noise format::names
# value_noise
# fbm3

shader_chunks list tag::noise format::json fields::name,stage
# [ { "name": "value_noise", "stage": "(none)" }, ... ]

shader_chunks list roots::1 fields::name,exports
# name                 exports
# -------------------  -------------------------------------------------
# fbm3                 fn fbm3(p: vec2f) -> f32
# fullscreen_triangle  struct VertexOutput { position: vec4f, uv: vec2f };
#                      fn vs_main(vertex_index: u32) -> VertexOutput

shader_chunks list depends_on::hash21 transitive::1 count::1
# 2

shader_chunks list format::bogus
# invalid `format` value: `bogus` (allowed: table, markdown, expanded, json, yaml, names)
# (exit 1)
```

### Notes
- Pipeline order is fixed: select (`names`) → filter → `count::`
  short-circuit → sort → page (`offset::`/`limit::`) → render.
- Read-only — only the compiled-in `shader_chunks_core::CHUNKS` table is
  consulted.
- An over-narrow filter yields empty output with exit 0; an *invalid
  value* (unknown field, unknown `depends_on::` chunk, bad enum spelling,
  negative integer) exits 1 loudly.
- Output formats: all 6 of [`../format/`](../format/readme.md)'s
  query-command formats, selected by `format::`.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.get`](02_get.md) | Same engine, names required, detail defaults |
| 2 | [`.tags`](03_tags.md) | Discover the `tag::` selector vocabulary |
| 3 | [`.tree`](04_tree.md) | Dependency *structure* instead of a filtered set |

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
