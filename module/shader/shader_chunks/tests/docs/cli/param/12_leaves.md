# Parameter Test :: leaves

Source: [`../../../../docs/cli/param/12_leaves.md`](../../../../docs/cli/param/12_leaves.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `leaves::1` selects exactly the chunks with no dependencies | `shader_chunks_test.rs::query_roots_and_leaves_select_graph_extremes` |
| EC-2 | Combined `roots::1 leaves::1` intersects to the fully isolated chunk | `shader_chunks_test.rs::query_roots_and_leaves_select_graph_extremes` |

### Simple Co-Dependencies

Dual of [`roots::`](11_roots.md) — outbound vs. inbound edges;
conjunctive with it and every other
[filtering](../param_group/01_filtering.md) member.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 2 |
| Real test functions | 1 (covers both cases) |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/07_switch.md`](../type/07_switch.md) | Underlying boolean coercion contract |
| [`../param/11_roots.md`](11_roots.md) | The dual selector |
