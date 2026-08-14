# Type Test :: SortOrder

Source: [`../../../../docs/cli/type/05_sort_order.md`](../../../../docs/cli/type/05_sort_order.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Round-trip: both variants survive `as_str` → `parse` | `shader_chunks_test.rs::query_enum_params_round_trip_and_reject_bogus_values` |
| TC-2 | Invalid-input rejection: bogus string fails to parse; loud exit end-to-end | `shader_chunks_test.rs::query_enum_params_round_trip_and_reject_bogus_values`; `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |
| TC-3 | Behavioral: `Desc` reverses every sort key's result — including `input` | `shader_chunks_test.rs::query_order_desc_reverses_including_input_order` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 3 |
| Real test functions referenced | 3 |
| Round-trip | TC-1 |
| Invalid-input rejection | TC-2 |
| Direction semantics | TC-3 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/17_order.md`](../param/17_order.md) | `order` parameter — the sole usage context |
