# Type Tests

### Scope

- **Purpose:** Construction, parsing, and rejection test specifications
  for every type this crate uses.
- **Responsibility:** One file per type, documenting the abstract
  contract independent of any single parameter's usage context.
- **In Scope:** The 10 types in [`../../../docs/cli/type/`](../../../docs/cli/type/readme.md).
- **Out of Scope:** Parameter-specific usage constraints (→ [`../param/`](../param/readme.md));
  the family's one remaining type,
  [`Float`](../../../../../shader_chunks_render/tests/docs/cli/type/01_float.md),
  owned by `shader_chunks_render`.

---

### Overview Table

| # | File | Type | Status |
|---|------|------|--------|
| 1 | [01_chunk_name.md](01_chunk_name.md) | ChunkName | ✅ |
| 2 | [02_field_name.md](02_field_name.md) | FieldName | ✅ |
| 3 | [03_output_format.md](03_output_format.md) | OutputFormat | ✅ |
| 4 | [04_sort_key.md](04_sort_key.md) | SortKey | ✅ |
| 5 | [05_sort_order.md](05_sort_order.md) | SortOrder | ✅ |
| 6 | [06_tags_mode.md](06_tags_mode.md) | TagsMode | ✅ |
| 7 | [07_switch.md](07_switch.md) | Switch | ✅ |
| 8 | [08_non_negative_integer.md](08_non_negative_integer.md) | NonNegativeInteger | ✅ |
| 9 | [09_tag_selector.md](09_tag_selector.md) | TagSelector | ✅ |
| 10 | [10_stage_selector.md](10_stage_selector.md) | StageSelector | ✅ |

**Total:** 10 type test specs (of 12 across the `shader_chunks` family)

The 4 genuine enums (OutputFormat, SortKey, SortOrder, TagsMode) share
one round-trip/rejection test
(`query_enum_params_round_trip_and_reject_bogus_values`) plus dedicated
behavioral tests; the open selectors (TagSelector, StageSelector)
deliberately have no rejection case — unmatched input yields empty
output with exit 0.

### Docs

| File | Relationship |
|------|--------------|
| [`../readme.md`](../readme.md) | Test tree root (this crate) |
| [`../../../docs/cli/type/readme.md`](../../../docs/cli/type/readme.md) | Type documentation source |
