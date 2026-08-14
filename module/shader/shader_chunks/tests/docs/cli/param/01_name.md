# Parameter Test :: name

Source: [`../../../../docs/cli/param/01_name.md`](../../../../docs/cli/param/01_name.md)

### Edge Cases

| ID | Case | Command | Real Test |
|----|------|---------|-----------|
| EC-1 | Valid, known chunk name | `tree` | `shader_chunks_test.rs::tree_chunk_shows_fbm3_dependency_chain_in_order`; `cli_subprocess_test.rs::tree_fbm3_shows_the_dependency_chain` |
| EC-2 | Absent (omitted entirely) | `tree` | `shader_chunks_test.rs::tree_chunk_with_no_name_shows_forest_of_every_root_chunk` |
| EC-3 | Unknown chunk name | `tree` | `shader_chunks_test.rs::tree_chunk_reports_unknown_chunk_error_for_bogus_name` |
| EC-4 | Valid chunk with zero declared tunables — explicit message, exit 0 | `tunables` | `shader_chunks_test.rs::tunables_zero_declared_params_reports_explicit_message_not_blank_or_error`; `cli_subprocess_test.rs::tunables_unannotated_real_chunk_prints_explicit_empty_message` |
| EC-5 | Unknown chunk name | `tunables` | `shader_chunks_test.rs::tunables_unknown_chunk_reports_unknown_chunk_error`; `cli_subprocess_test.rs::tunables_bogus_chunk_exits_non_zero_without_a_panic_backtrace` |

### Simple Co-Dependencies

`tree` (optional — omission selects forest mode rather than erroring) and
`tunables` (required — no omission form) are the two commands declaring
`name`; `get` moved to the plural [`names`](02_names.md) when it adopted
the shared query engine. No other parameter interacts with `name`.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 5 |
| Real test functions | 8 |
| P0 (exit-code-affecting) | EC-3, EC-5 |
| P1 (structural output) | EC-1, EC-2, EC-4 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_004_tree.md`](../command/cmd_004_tree.md) | Command using `name` as optional |
| [`../command/cmd_006_tunables.md`](../command/cmd_006_tunables.md) | Command using `name` as required |
| [`../type/01_chunk_name.md`](../type/01_chunk_name.md) | Underlying type contract |
