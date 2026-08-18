# Types

### Scope

- **Purpose:** Documents every semantic parameter type this crate uses.
- **Responsibility:** One dedicated file per type — purpose, fundamental
  representation, constraints, parsing, methods.
- **In Scope:** The 11 semantic types behind this crate's 24 parameters —
  5 genuine Rust enums (`shader_chunks_query_core/src/lib.rs`) and 6
  string/bool/usize-realized types whose constraints live in validation
  code.
- **Out of Scope:** Per-parameter defaults/requiredness (→ [`../param/`](../param/readme.md)),
  the family's one remaining type — [`Float`](../../../../shader_chunks_render/docs/cli/type/01_float.md),
  owned by `shader_chunks_render` (→
  [family index](../../../../shader_chunks/docs/cli/readme.md)).

---

### Overview Table

| # | File | Type | Fundamental | Status |
|---|------|------|-------------|--------|
| 1 | [01_chunk_name.md](01_chunk_name.md) | ChunkName | `String` | ✅ |
| 2 | [02_field_name.md](02_field_name.md) | FieldName | `String` (closed set of 7) | ✅ |
| 3 | [03_output_format.md](03_output_format.md) | OutputFormat | `enum` (6 variants) | ✅ |
| 4 | [04_sort_key.md](04_sort_key.md) | SortKey | `enum` (4 variants) | ✅ |
| 5 | [05_sort_order.md](05_sort_order.md) | SortOrder | `enum` (2 variants) | ✅ |
| 6 | [06_tags_mode.md](06_tags_mode.md) | TagsMode | `enum` (2 variants) | ✅ |
| 7 | [07_switch.md](07_switch.md) | Switch | `bool` | ✅ |
| 8 | [08_non_negative_integer.md](08_non_negative_integer.md) | NonNegativeInteger | `usize` | ✅ |
| 9 | [09_tag_selector.md](09_tag_selector.md) | TagSelector | `String` (pair or bare) | ✅ |
| 10 | [10_stage_selector.md](10_stage_selector.md) | StageSelector | `String` (any/none/literal) | ✅ |
| 11 | [11_tree_format.md](11_tree_format.md) | TreeFormat | `enum` (3 variants) | ✅ |

**Total:** 11 types (of 13 across the `shader_chunks` family)

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root (this crate) |
| [../param/readme.md](../param/readme.md) | Parameters carrying this type |
| [../command/readme.md](../command/readme.md) | Commands using this type via a parameter |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/type/readme.md](../../../tests/docs/cli/type/readme.md) | Type-level test specifications |
