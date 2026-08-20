# Parameter Test :: order

Source: [`../../../../docs/cli/param/17_order.md`](../../../../docs/cli/param/17_order.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `desc` reverses every sort key — including `input` order | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_order_desc_reverses_including_input_order` |
| EC-2 | Both values round-trip through `parse`/`as_str` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_enum_params_round_trip_and_reject_bogus_values` |
| EC-3 | Bogus value rejected loudly end-to-end | `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |

### Simple Co-Dependencies

A pure modifier of [`sort::`](16_sort.md) — meaningful for every key,
since even `input` order can be reversed (EC-1).

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
| [`../type/05_sort_order.md`](../type/05_sort_order.md) | Underlying enum contract |
| [`../param/16_sort.md`](16_sort.md) | The key this direction applies to |
