# Types

### Scope

- **Purpose:** Documents every semantic parameter type `shader_chunks` uses.
- **Responsibility:** One dedicated file per type — purpose, fundamental representation, constraints, parsing, methods.
- **In Scope:** The 10 semantic types behind the 21 parameters — 4 genuine Rust enums (`src/lib.rs`) and 6 string/bool/usize-realized types whose constraints live in validation code.
- **Out of Scope:** Per-parameter defaults/requiredness (→ [`../param/`](../param/readme.md)).

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

**Total:** 10 types

### Docs

| File | Relationship |
|------|--------------|
| [../readme.md](../readme.md) | CLI documentation root |
| [../param/readme.md](../param/readme.md) | Parameters carrying this type |
| [../command/readme.md](../command/readme.md) | Commands using this type via a parameter |

### Tests

| File | Relationship |
|------|--------------|
| [../../../tests/docs/cli/type/readme.md](../../../tests/docs/cli/type/readme.md) | Type-level test specifications |
