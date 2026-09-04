# Parameter Test :: count

Source: [`../../../../docs/cli/param/14_count.md`](../../../../docs/cli/param/14_count.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Reports the filtered total, computed before `limit::`/`offset::` paging | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_count_reports_filtered_total_before_paging` |
| EC-2 | End-to-end: bare `count::1` prints `20` and nothing else | `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` |

### Simple Co-Dependencies

Short-circuits the pipeline: [`fields::`](13_fields.md),
[`format::`](15_format.md), [`sort::`](16_sort.md), and the
[paging pair](18_limit.md) are all irrelevant once `count::1` is set.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 2 |
| Real test functions | 2 |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/07_switch.md`](../type/07_switch.md) | Underlying boolean coercion contract |
| [`../param_group/02_projection.md`](../param_group/02_projection.md) | Short-circuit position in the pipeline |
