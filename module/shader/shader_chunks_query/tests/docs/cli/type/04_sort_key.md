# Type Test :: SortKey

Source: [`../../../../docs/cli/type/04_sort_key.md`](../../../../docs/cli/type/04_sort_key.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Round-trip: all 4 variants survive `as_str` → `parse` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_enum_params_round_trip_and_reject_bogus_values` |
| TC-2 | Invalid-input rejection: bogus string fails to parse; loud exit end-to-end | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_enum_params_round_trip_and_reject_bogus_values`; `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |
| TC-3 | Behavioral: each variant produces its documented deterministic ordering | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_sort_keys_order_deterministically` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 3 |
| Real test functions referenced | 3 |
| Round-trip | TC-1 |
| Invalid-input rejection | TC-2 |
| Ordering semantics | TC-3 (all 4 variants exercised) |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/16_sort.md`](../param/16_sort.md) | `sort` parameter — the sole usage context |
