# Parameter Test :: tags_mode

Source: [`../../../../docs/cli/param/06_tags_mode.md`](../../../../docs/cli/param/06_tags_mode.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `any` unions the same selector list `all` intersects | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_tags_mode_any_unions_and_all_intersects_selectors` |
| EC-2 | Round-trip of both spellings; bogus value rejects loudly | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_enum_params_round_trip_and_reject_bogus_values` |
| EC-3 | Bogus value exits non-zero through the CLI with the allowed set named | `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |

### Simple Co-Dependencies

Only observable with ≥2 [`tag::`](05_tag.md) selectors.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 3 |
| Real test functions | 3 |
| P0 (exit-code-affecting) | EC-2, EC-3 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/06_tags_mode.md`](../type/06_tags_mode.md) | Enum parsing contract |
| [`../param_group/01_filtering.md`](../param_group/01_filtering.md) | Interaction with `tag::` |
