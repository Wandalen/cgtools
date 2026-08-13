# Parameter Test :: stage

Source: [`../../../../docs/cli/param/07_stage.md`](../../../../docs/cli/param/07_stage.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `none` selects only stage-less chunks | `shader_chunks_test.rs::query_stage_filter_selects_none_literal_and_any` |
| EC-2 | Literal (`vertex`) selects exactly the declaring chunk | `shader_chunks_test.rs::query_stage_filter_selects_none_literal_and_any` |
| EC-3 | Unmatched literal (`fragment`) yields empty output, exit 0 — not an error | `shader_chunks_test.rs::query_stage_filter_selects_none_literal_and_any` |

### Simple Co-Dependencies

None — an independent [filtering](../param_group/01_filtering.md) member;
`any` (the default) disables it.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 3 |
| Real test functions | 1 (covers all three selector arms) |
| P1 (structural output) | EC-1, EC-2, EC-3 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/10_stage_selector.md`](../type/10_stage_selector.md) | Three-way selector contract |
