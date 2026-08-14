# Parameter Test :: case

Source: [`../../../../docs/cli/param/04_case.md`](../../../../docs/cli/param/04_case.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `case::1` flips a previously-matching insensitive `pattern::` to non-matching | `shader_chunks_test.rs::query_pattern_with_case_switch_demands_exact_case` |
| EC-2 | `case::` toggles `exports::` sensitivity with the same switch | `shader_chunks_test.rs::query_exports_filter_matches_signatures_with_case_switch` |

### Simple Co-Dependencies

A pure modifier of [`pattern::`](03_pattern.md) and
[`exports::`](10_exports.md); a no-op with neither set (covered
implicitly by every defaults test).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 2 |
| Real test functions | 2 |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../param_group/01_filtering.md`](../param_group/01_filtering.md) | Both modifier pairs |
| [`../type/07_switch.md`](../type/07_switch.md) | Underlying boolean coercion contract |
