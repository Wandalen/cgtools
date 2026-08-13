# Parameter Test :: exports

Source: [`../../../../docs/cli/param/10_exports.md`](../../../../docs/cli/param/10_exports.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Mixed-case needle matches signatures insensitively by default | `shader_chunks_test.rs::query_exports_filter_matches_signatures_with_case_switch` |
| EC-2 | Same needle under `case::1` — exact-case demanded, match set shrinks | `shader_chunks_test.rs::query_exports_filter_matches_signatures_with_case_switch` |

### Simple Co-Dependencies

Modified by [`case::`](04_case.md) — the same switch that governs
[`pattern::`](03_pattern.md).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 2 |
| Real test functions | 1 (covers both switch states) |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../param_group/01_filtering.md`](../param_group/01_filtering.md) | Interaction with `case::` |
