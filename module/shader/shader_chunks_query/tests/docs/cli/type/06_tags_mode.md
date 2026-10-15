# Type Test :: TagsMode

Source: [`../../../../docs/cli/type/06_tags_mode.md`](../../../../docs/cli/type/06_tags_mode.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Round-trip: both variants survive `as_str` → `parse` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_enum_params_round_trip_and_reject_bogus_values` |
| TC-2 | Invalid-input rejection: bogus string fails to parse; loud exit end-to-end | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_enum_params_round_trip_and_reject_bogus_values`; `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |
| TC-3 | Behavioral: `Any` unions the tag selectors, `All` intersects them | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_tags_mode_any_unions_and_all_intersects_selectors` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 3 |
| Real test functions referenced | 3 |
| Round-trip | TC-1 |
| Invalid-input rejection | TC-2 |
| Combination semantics | TC-3 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/06_tags_mode.md`](../param/06_tags_mode.md) | `tags_mode` parameter — the sole usage context |
| [`../type/09_tag_selector.md`](09_tag_selector.md) | The selectors this mode combines |
