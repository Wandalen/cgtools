# Type Test :: NonNegativeInteger

Source: [`../../../../docs/cli/type/08_non_negative_integer.md`](../../../../docs/cli/type/08_non_negative_integer.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Positive value: slices/truncates as documented | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_offset_and_limit_page_the_result` (`limit`, `offset`); `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_markdown_format_renders_pipe_table_with_heading_and_width` (`width`) |
| TC-2 | Zero (reserved default): unlimited / no-skip / auto-width | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_list_defaults_renders_every_chunk_as_plain_table` |
| TC-3 | Negative rejection: `cli.rs::arg_usize` maps `usize::try_from` failure to `CliError::InvalidParam` (`allowed : "a non-negative integer"`), exit 1 — one shared routine for `limit`/`offset`/`width` | `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` (`limit::-1`) |

TC-3's subprocess case drives the single shared `arg_usize` routine; a
defect in it would fail all three parameters identically, which is why
one end-to-end negative case suffices at this tier.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 3 |
| Real test functions referenced | 4 |
| Positive semantics | TC-1 |
| Zero-reserved semantics | TC-2 |
| Invalid-input rejection | TC-3 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/18_limit.md`](../param/18_limit.md) | Usage: keep-at-most bound |
| [`../param/19_offset.md`](../param/19_offset.md) | Usage: skip-first bound |
| [`../param/21_width.md`](../param/21_width.md) | Usage: column truncation bound |
