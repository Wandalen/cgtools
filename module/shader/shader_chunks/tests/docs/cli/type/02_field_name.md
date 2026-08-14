# Type Test :: FieldName

Source: [`../../../../docs/cli/type/02_field_name.md`](../../../../docs/cli/type/02_field_name.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Construction: each member of the closed 7-name set selects its column | `shader_chunks_test.rs::query_fields_projects_only_the_named_columns` |
| TC-2 | Exhaustiveness: all 7 declared fields render — including multi-line `source` | `shader_chunks_test.rs::query_every_declared_field_renders_including_source` |
| TC-3 | Invalid-input rejection: a name outside the set is `CliError::UnknownField`, exit 1, error text listing the valid names | `shader_chunks_test.rs::query_unknown_field_fails_loudly`; `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |

`FieldName` has no round-trip case of its own: the input string *is* the
column header rendered in table/markdown output, so TC-1's projection
assertion already pins identity.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 3 |
| Real test functions referenced | 4 |
| Construction/parsing | TC-1, TC-2 |
| Invalid-input rejection | TC-3 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/13_fields.md`](../param/13_fields.md) | `fields` parameter — the sole usage context |
