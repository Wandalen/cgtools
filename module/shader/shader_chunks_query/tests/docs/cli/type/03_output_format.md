# Type Test :: OutputFormat

Source: [`../../../../docs/cli/type/03_output_format.md`](../../../../docs/cli/type/03_output_format.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Round-trip: all 6 variants survive `as_str` → `parse` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_enum_params_round_trip_and_reject_bogus_values` |
| TC-2 | Invalid-input rejection: bogus string is `CliError::InvalidParam { param : "format" }`, exit 1 | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_enum_params_round_trip_and_reject_bogus_values`; `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |
| TC-3 | Behavioral: each parsed variant drives its distinct renderer | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_json_and_yaml_formats_carry_row_content`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_markdown_format_renders_pipe_table_with_heading_and_width`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_names_format_ignores_fields_projection`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_get_defaults_renders_expanded_records_with_detail_fields`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_list_defaults_renders_every_chunk_as_plain_table` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 3 |
| Real test functions referenced | 7 |
| Round-trip | TC-1 |
| Invalid-input rejection | TC-2 |
| Renderer dispatch | TC-3 (all 6 variants exercised) |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/15_format.md`](../param/15_format.md) | `format` parameter — the sole usage context |
| [`../../../../docs/cli/format/readme.md`](../../../../docs/cli/format/readme.md) | Per-variant structural specs |
