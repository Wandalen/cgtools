# Parameter Test :: fields

Source: [`../../../../docs/cli/param/13_fields.md`](../../../../docs/cli/param/13_fields.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Projection keeps only the named columns, in the given order | `shader_chunks_test.rs::query_fields_projects_only_the_named_columns` |
| EC-2 | Every declared field renders — including multi-line `source` | `shader_chunks_test.rs::query_every_declared_field_renders_including_source` |
| EC-3 | Unknown field fails loudly (`CliError::UnknownField`) | `shader_chunks_test.rs::query_unknown_field_fails_loudly`; `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |
| EC-4 | Identical `fields::` on both commands — byte-identical output | `cli_subprocess_test.rs::list_and_get_agree_under_identical_explicit_parameters` |

### Simple Co-Dependencies

Ignored entirely by [`format::names`](15_format.md) and pre-empted by
[`count::1`](14_count.md) — projection only matters when rows render.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 4 |
| Real test functions | 5 |
| P0 (exit-code-affecting) | EC-3 |
| P1 (structural output) | EC-1, EC-2, EC-4 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/02_field_name.md`](../type/02_field_name.md) | Underlying type contract |
| [`../param_group/02_projection.md`](../param_group/02_projection.md) | Group-level interaction rules |
