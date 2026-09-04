# Parameter Test :: heading

Source: [`../../../../docs/cli/param/20_heading.md`](../../../../docs/cli/param/20_heading.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Non-empty heading renders as a rule line above the markdown table | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_markdown_format_renders_pipe_table_with_heading_and_width` |
| EC-2 | Empty (default): no heading line emitted | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_list_defaults_renders_every_chunk_as_plain_table` |

### Simple Co-Dependencies

Consumed only by the table/markdown renderers selected via
[`format::`](15_format.md) — silently ignored by expanded, json, yaml,
and names (a documented no-op, not an error).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 2 |
| Real test functions | 2 |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../../../../docs/cli/format/04_markdown.md`](../../../../docs/cli/format/04_markdown.md) | Heading rendering shape |
| [`../param_group/03_formatting.md`](../param_group/03_formatting.md) | Table/markdown-only no-op rule |
