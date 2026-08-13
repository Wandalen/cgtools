# Parameter Test :: offset

Source: [`../../../../docs/cli/param/19_offset.md`](../../../../docs/cli/param/19_offset.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `offset::N` skips the first N rows before `limit::` applies | `shader_chunks_test.rs::query_offset_and_limit_page_the_result` |
| EC-2 | Offset past the end yields empty output, exit 0 — not an error | `shader_chunks_test.rs::query_offset_and_limit_page_the_result` |
| EC-3 | Negative value rejected via the shared `arg_usize` routine — exercised end-to-end through its `limit::-1` sibling | `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |

### Simple Co-Dependencies

Paired with [`limit::`](18_limit.md); applied after
[`sort::`](16_sort.md)/[`order::`](17_order.md) so pages are stable
under a fixed ordering.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 3 |
| Real test functions | 2 |
| P0 (exit-code-affecting) | EC-3 (shared-routine evidence) |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/08_non_negative_integer.md`](../type/08_non_negative_integer.md) | Underlying integer contract (`arg_usize`) |
| [`../param/18_limit.md`](18_limit.md) | The paired slice bound |
