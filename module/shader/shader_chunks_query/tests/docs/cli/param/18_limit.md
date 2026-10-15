# Parameter Test :: limit

Source: [`../../../../docs/cli/param/18_limit.md`](../../../../docs/cli/param/18_limit.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `limit::N` keeps at most N rows, applied after sort and offset | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_offset_and_limit_page_the_result` |
| EC-2 | `0` means unlimited (the default) | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_list_defaults_renders_every_chunk_as_plain_table` |
| EC-3 | Negative value rejected loudly (`arg_usize`), exit 1 | `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |

### Simple Co-Dependencies

Paired with [`offset::`](19_offset.md) (skip-then-keep);
[`count::`](14_count.md) reports the total from *before* this slice.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 3 |
| Real test functions | 3 |
| P0 (exit-code-affecting) | EC-3 |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/08_non_negative_integer.md`](../type/08_non_negative_integer.md) | Underlying integer contract (`arg_usize`) |
| [`../param_group/03_formatting.md`](../param_group/03_formatting.md) | Sort→page→render pipeline position |
