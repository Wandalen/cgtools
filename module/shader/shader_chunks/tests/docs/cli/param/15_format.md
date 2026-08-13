# Parameter Test :: format

Source: [`../../../../docs/cli/param/15_format.md`](../../../../docs/cli/param/15_format.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | All six values round-trip through `parse`/`as_str` | `shader_chunks_test.rs::query_enum_params_round_trip_and_reject_bogus_values` |
| EC-2 | Bogus value rejected as `InvalidParam { param : "format" }`, exit 1 | `shader_chunks_test.rs::query_enum_params_round_trip_and_reject_bogus_values`; `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |
| EC-3 | `json` and `yaml` carry full row content (parsed, not string-matched) | `shader_chunks_test.rs::query_json_and_yaml_formats_carry_row_content` |
| EC-4 | `markdown` renders a pipe table honoring `heading::`/`width::` | `shader_chunks_test.rs::query_markdown_format_renders_pipe_table_with_heading_and_width` |
| EC-5 | `names` emits one name per line, ignoring `fields::` | `shader_chunks_test.rs::query_names_format_ignores_fields_projection`; `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` |
| EC-6 | `expanded` via explicit `format::` on `list` equals `get`'s default | `cli_subprocess_test.rs::list_and_get_agree_under_identical_explicit_parameters` |

### Simple Co-Dependencies

Selects which renderer consumes [`fields::`](13_fields.md),
[`heading::`](20_heading.md), and [`width::`](21_width.md) — the latter
two are no-ops outside table/markdown.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 6 |
| Real test functions | 7 |
| P0 (exit-code-affecting) | EC-2 |
| P1 (structural output) | EC-1, EC-3, EC-4, EC-5, EC-6 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/03_output_format.md`](../type/03_output_format.md) | Underlying enum contract |
| [`../../../../docs/cli/format/readme.md`](../../../../docs/cli/format/readme.md) | Per-format structural specs under test |
