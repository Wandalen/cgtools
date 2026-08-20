# Parameter Test :: roots

Source: [`../../../../docs/cli/param/11_roots.md`](../../../../docs/cli/param/11_roots.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `roots::1` selects exactly the chunks nothing else depends on | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_roots_and_leaves_select_graph_extremes` |
| EC-2 | Combined `roots::1 leaves::1` intersects to the fully isolated chunk | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_roots_and_leaves_select_graph_extremes` |
| EC-3 | End-to-end through the CLI binding | `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` |

### Simple Co-Dependencies

Conjunctive with [`leaves::`](12_leaves.md) and every other
[filtering](../param_group/01_filtering.md) member.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 3 |
| Real test functions | 2 |
| P1 (structural output) | EC-1, EC-2, EC-3 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/07_switch.md`](../type/07_switch.md) | Underlying boolean coercion contract |
| [`../command_group/02_graph.md`](../command_group/02_graph.md) | `tree`'s forest renders these same roots |
