# Parameter Test :: width

Source: [`../../../../docs/cli/param/21_width.md`](../../../../docs/cli/param/21_width.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `width::N` truncates over-long cells with `...` in table/markdown | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_markdown_format_renders_pipe_table_with_heading_and_width` |
| EC-2 | `0` (default): automatic column sizing | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_list_defaults_renders_every_chunk_as_plain_table` |
| EC-3 | Negative value rejected via the shared `arg_usize` routine — exercised end-to-end through its `limit::-1` sibling | `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |

### Simple Co-Dependencies

Consumed only by the table/markdown renderers selected via
[`format::`](15_format.md); silently ignored elsewhere, mirroring
[`heading::`](20_heading.md).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 3 |
| Real test functions | 3 |
| P0 (exit-code-affecting) | EC-3 (shared-routine evidence) |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/08_non_negative_integer.md`](../type/08_non_negative_integer.md) | Underlying integer contract (`arg_usize`) |
| [`../param_group/03_formatting.md`](../param_group/03_formatting.md) | Table/markdown-only no-op rule |
