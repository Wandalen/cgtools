# Parameter Test :: name

Source: [`../../../../docs/cli/param/01_name.md`](../../../../docs/cli/param/01_name.md)

### Edge Cases

| ID | Case | Command | Real Test |
|----|------|---------|-----------|
| EC-1 | Valid, known chunk name | `get` | `shader_chunks_test.rs::get_chunk_reports_full_detail_for_hash21`; `cli_subprocess_test.rs::get_hash21_prints_full_detail` |
| EC-2 | Unknown chunk name | `get` | `shader_chunks_test.rs::get_chunk_reports_unknown_chunk_error_for_bogus_name`; `cli_subprocess_test.rs::get_unknown_chunk_exits_non_zero_without_a_panic_backtrace` |
| EC-3 | Valid, known chunk name | `tree` | `shader_chunks_test.rs::tree_chunk_shows_fbm3_dependency_chain_in_order`; `cli_subprocess_test.rs::tree_fbm3_shows_the_dependency_chain` |
| EC-4 | Absent (omitted entirely) | `tree` | `shader_chunks_test.rs::tree_chunk_with_no_name_shows_forest_of_every_root_chunk` |
| EC-5 | Unknown chunk name | `tree` | `shader_chunks_test.rs::tree_chunk_reports_unknown_chunk_error_for_bogus_name` |

### Simple Co-Dependencies

`name`'s requiredness is determined entirely by which command declares it:
`get` treats it as required (EC-1/EC-2 — omitting it is a `unilang`
dispatch error, never reaches `CliError`); `tree` treats it as optional
(EC-3/EC-4/EC-5 — omission selects forest mode rather than erroring). No
other parameter interacts with `name`.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 5 |
| Real test functions | 6 (2 shared across cases) |
| P0 (exit-code-affecting) | EC-2, EC-5 |
| P1 (structural output) | EC-1, EC-3, EC-4 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_002_get.md`](../command/cmd_002_get.md) | Command using `name` as required |
| [`../command/cmd_004_tree.md`](../command/cmd_004_tree.md) | Command using `name` as optional |
| [`../type/01_chunk_name.md`](../type/01_chunk_name.md) | Underlying type contract |
