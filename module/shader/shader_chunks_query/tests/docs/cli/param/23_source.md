# Parameter Test :: source

Source: [`../../../../docs/cli/param/23_source.md`](../../../../docs/cli/param/23_source.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Needle matches raw WGSL body text (not just names/exports) insensitively by default | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_source_filter_matches_wgsl_body_text_with_case_switch` |
| EC-2 | Same needle under `case::1` — exact-case demanded, match set shrinks | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_source_filter_matches_wgsl_body_text_with_case_switch` |

### Simple Co-Dependencies

Modified by [`case::`](04_case.md) — the same switch that governs
[`pattern::`](03_pattern.md) and [`exports::`](10_exports.md).

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
