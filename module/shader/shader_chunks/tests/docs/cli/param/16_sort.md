# Parameter Test :: sort

Source: [`../../../../docs/cli/param/16_sort.md`](../../../../docs/cli/param/16_sort.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Each key (`input`/`name`/`stage`/`description`) orders deterministically | `shader_chunks_test.rs::query_sort_keys_order_deterministically` |
| EC-2 | All four values round-trip through `parse`/`as_str` | `shader_chunks_test.rs::query_enum_params_round_trip_and_reject_bogus_values` |
| EC-3 | Bogus value rejected loudly end-to-end | `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |

### Simple Co-Dependencies

Direction supplied by [`order::`](17_order.md); sorting happens before
the [paging pair](18_limit.md) slices.

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
| [`../type/04_sort_key.md`](../type/04_sort_key.md) | Underlying enum contract |
| [`../param_group/03_formatting.md`](../param_group/03_formatting.md) | Sort→page→render pipeline position |
