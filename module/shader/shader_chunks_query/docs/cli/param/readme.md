# Parameters

### Scope

- **Purpose:** Documents every parameter accepted by this crate's commands.
- **Responsibility:** One dedicated file per parameter, unified across
  every command that accepts it.
- **In Scope:** The 22 parameters declared across `list`/`get`/`tags`/`tree`'s
  `ArgumentDefinition`s — 2 positional selectors (`name`, `names`) plus
  the 19 named query parameters `.list` and `.get` share verbatim (one of
  which, `transitive::`, `.compose` in the `shader_chunks_compose` crate
  also accepts as its closure switch), plus `.tree`'s own `reverse::`
  switch.
- **Out of Scope:** Command-level syntax/examples (→ [`../command/`](../command/readme.md)),
  type constraints/parsing (→ [`../type/`](../type/readme.md)),
  co-occurrence/interaction rules (→ [`../param_group/`](../param_group/readme.md)),
  the 5 remaining parameters of the `shader_chunks` family — `file`,
  `serve` (preview), `out`, `size`, `time` (render) — (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Parameter | Type | Default | Status |
|---|------|-----------|------|---------|--------|
| 1 | [01_name.md](01_name.md) | `name` | [`ChunkName`](../type/01_chunk_name.md) | `Varies` — optional for `tree` (omit for the forest); also accepted (required) by `.tunables`/`.preview`/`.render` in their own crates | ✅ |
| 2 | [02_names.md](02_names.md) | `names` | [`ChunkName`](../type/01_chunk_name.md) (list) | `Varies` — required for `get`/`compose`, optional for `list` | ✅ |
| 3 | [03_pattern.md](03_pattern.md) | `pattern` | String | off | ✅ |
| 4 | [04_case.md](04_case.md) | `case` | [`Switch`](../type/07_switch.md) | `false` | ✅ |
| 5 | [05_tag.md](05_tag.md) | `tag` | [`TagSelector`](../type/09_tag_selector.md) (list) | off | ✅ |
| 6 | [06_tags_mode.md](06_tags_mode.md) | `tags_mode` | [`TagsMode`](../type/06_tags_mode.md) | `any` | ✅ |
| 7 | [07_stage.md](07_stage.md) | `stage` | [`StageSelector`](../type/10_stage_selector.md) | `any` | ✅ |
| 8 | [08_depends_on.md](08_depends_on.md) | `depends_on` | [`ChunkName`](../type/01_chunk_name.md) | off | ✅ |
| 9 | [09_transitive.md](09_transitive.md) | `transitive` | [`Switch`](../type/07_switch.md) | `false` | ✅ |
| 10 | [10_exports.md](10_exports.md) | `exports` | String | off | ✅ |
| 11 | [11_roots.md](11_roots.md) | `roots` | [`Switch`](../type/07_switch.md) | `false` | ✅ |
| 12 | [12_leaves.md](12_leaves.md) | `leaves` | [`Switch`](../type/07_switch.md) | `false` | ✅ |
| 13 | [13_fields.md](13_fields.md) | `fields` | [`FieldName`](../type/02_field_name.md) (list) | `Varies` per command | ✅ |
| 14 | [14_count.md](14_count.md) | `count` | [`Switch`](../type/07_switch.md) | `false` | ✅ |
| 15 | [15_format.md](15_format.md) | `format` | [`OutputFormat`](../type/03_output_format.md) | `Varies` per command | ✅ |
| 16 | [16_sort.md](16_sort.md) | `sort` | [`SortKey`](../type/04_sort_key.md) | `input` | ✅ |
| 17 | [17_order.md](17_order.md) | `order` | [`SortOrder`](../type/05_sort_order.md) | `asc` | ✅ |
| 18 | [18_limit.md](18_limit.md) | `limit` | [`NonNegativeInteger`](../type/08_non_negative_integer.md) | `0` (unlimited) | ✅ |
| 19 | [19_offset.md](19_offset.md) | `offset` | [`NonNegativeInteger`](../type/08_non_negative_integer.md) | `0` | ✅ |
| 20 | [20_heading.md](20_heading.md) | `heading` | String | off | ✅ |
| 21 | [21_width.md](21_width.md) | `width` | [`NonNegativeInteger`](../type/08_non_negative_integer.md) | `0` (auto) | ✅ |
| 22 | [22_reverse.md](22_reverse.md) | `reverse` | [`Switch`](../type/07_switch.md) | `false` | ✅ |

**Total:** 22 parameters (of 28 across the `shader_chunks` family)

**Parameter Groups:** the 19 named query parameters (#3–#21) partition
into 3 groups — [filtering](../param_group/01_filtering.md),
[projection](../param_group/02_projection.md),
[formatting](../param_group/03_formatting.md) — shared verbatim by
`.list` and `.get`; see [`../param_group/`](../param_group/readme.md).
The positional selectors (#1, #2) and `.tree`'s own `reverse` (#22)
belong to no group.

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../command/readme.md](../command/readme.md) | Commands accepting these parameters |
| [../type/readme.md](../type/readme.md) | Type definitions |
| [../param_group/readme.md](../param_group/readme.md) | Co-occurrence groups over these parameters |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/param/readme.md](../../../tests/docs/cli/param/readme.md) | Parameter-level test specifications |
