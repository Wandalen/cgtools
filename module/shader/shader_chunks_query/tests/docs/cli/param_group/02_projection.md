# Parameter Group Test :: projection

Source: [`../../../../docs/cli/param_group/02_projection.md`](../../../../docs/cli/param_group/02_projection.md)

### Group Cases (GRP-N)

| ID | Interaction | Real Test |
|----|-------------|-----------|
| GRP-1 | `count::1` short-circuits paging — the total is taken before `limit::` applies | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_count_reports_filtered_total_before_paging` |
| GRP-2 | `count::1` reflects the *filtered* set — a `pattern::` filter changes the reported number | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_count_reports_filtered_total_before_paging` (second assertion); `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` (count case) |
| GRP-3 | `fields::` interacts with `format::names` — the projection is ignored, names render regardless | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_names_format_ignores_fields_projection` |
| GRP-4 | Per-command `fields::` defaults differ while explicit `fields::` equalizes `list` and `get` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_list_defaults_renders_every_chunk_as_plain_table`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_get_defaults_renders_expanded_records_with_detail_fields`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_list_and_get_defaults_share_engine_and_agree_under_equal_params` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 4 |
| Real test functions | 5 |
| Short-circuit rules covered | 2 (`count` vs paging, `names` vs `fields`) |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../param/13_fields.md`](../param/13_fields.md) | `fields` parameter edge cases |
| [`../param/14_count.md`](../param/14_count.md) | `count` parameter edge cases |
| [`../command_group/01_query.md`](../command_group/01_query.md) | The command group whose engine this group projects |
