# Parameter Group Test :: formatting

Source: [`../../../../docs/cli/param_group/03_formatting.md`](../../../../docs/cli/param_group/03_formatting.md)

### Group Cases (GRP-N)

| ID | Interaction | Real Test |
|----|-------------|-----------|
| GRP-1 | `order::desc` reverses whichever `sort::` key is active, including `input` | `shader_chunks_test.rs::query_order_desc_reverses_including_input_order` |
| GRP-2 | `offset::` and `limit::` page the *sorted* sequence together; past-the-end offset yields empty output | `shader_chunks_test.rs::query_offset_and_limit_page_the_result` |
| GRP-3 | `heading::` and `width::` shape the `markdown` rendering together (heading rule + `...` truncation) | `shader_chunks_test.rs::query_markdown_format_renders_pipe_table_with_heading_and_width` |
| GRP-4 | Every enum member (`format`, `sort`, `order`, plus filtering's `tags_mode`) rejects bogus values loudly through one shared error shape | `shader_chunks_test.rs::query_enum_params_round_trip_and_reject_bogus_values`; `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |
| GRP-5 | Negative integers reject loudly across `limit::`/`offset::`/`width::` | `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` (negative cases) |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 5 |
| Real test functions | 5 |
| Pipeline-order rules covered | 2 (sort→page, render modifiers) |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../param/readme.md`](../param/readme.md) | Member parameters' own edge cases |
| [`../type/readme.md`](../type/readme.md) | Enum type parsing contracts |
| [`../command_group/01_query.md`](../command_group/01_query.md) | The command group whose engine this group shapes |
