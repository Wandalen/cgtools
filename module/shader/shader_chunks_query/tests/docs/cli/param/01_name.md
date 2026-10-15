# Parameter Test :: name

Source: [`../../../../docs/cli/param/01_name.md`](../../../../docs/cli/param/01_name.md)

### Edge Cases

| ID | Case | Command | Real Test |
|----|------|---------|-----------|
| EC-1 | Valid, known chunk name | `tree` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_shows_fbm3_dependency_chain_in_order`; `cli_subprocess_test.rs::tree_fbm3_shows_the_dependency_chain` |
| EC-2 | Absent (omitted entirely) | `tree` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_with_no_name_shows_forest_of_every_root_chunk` |
| EC-3 | Unknown chunk name | `tree` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_reports_unknown_chunk_error_for_bogus_name` |
| EC-4 | Valid chunk with zero declared tunables — explicit message, exit 0 | `tunables` | `shader_chunks_params/tests/tunables_test.rs::tunables_zero_declared_params_reports_explicit_message_not_blank_or_error`; `cli_subprocess_test.rs::tunables_unannotated_real_chunk_prints_explicit_empty_message` |
| EC-5 | Unknown chunk name | `tunables` | `shader_chunks_params/tests/tunables_test.rs::tunables_unknown_chunk_reports_unknown_chunk_error`; `cli_subprocess_test.rs::tunables_bogus_chunk_exits_non_zero_without_a_panic_backtrace` |
| EC-6 | Unknown chunk name | `preview` | `shader_chunks_preview/tests/preview_cli_test.rs::unknown_name_is_rejected_with_the_shared_unknown_chunk_text` |
| EC-7 | Unknown chunk name | `render` | `shader_chunks_render/tests/render_cli_test.rs::unknown_name_is_rejected_with_the_shared_unknown_chunk_text` |

### Simple Co-Dependencies

`tree` (optional — omission selects forest mode rather than erroring)
and `tunables` (required — no omission form) declare `name` standalone;
`preview` and `render` declare it as one of two mutually exclusive
targets (exactly one of `name`/`file::` is required —
their target-arm cases live in
[`file`](../../../../../shader_chunks_preview/tests/docs/cli/param/01_file.md) and the two
command specs). `get` moved to the plural [`names`](02_names.md) when it
adopted the shared query engine. No other parameter interacts with
`name`.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 7 |
| Real test functions | 10 |
| P0 (exit-code-affecting) | EC-3, EC-5, EC-6, EC-7 |
| P1 (structural output) | EC-1, EC-2, EC-4 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_004_tree.md`](../command/cmd_004_tree.md) | Command using `name` as optional |
| [`tunables`](../../../../../shader_chunks_params/tests/docs/cli/command/cmd_001_tunables.md) | Command using `name` as required |
| [`preview`](../../../../../shader_chunks_preview/tests/docs/cli/command/cmd_001_preview.md) | Command using `name` as one of two exclusive targets |
| [`render`](../../../../../shader_chunks_render/tests/docs/cli/command/cmd_001_render.md) | Command using `name` as one of two exclusive targets |
| [`../type/01_chunk_name.md`](../type/01_chunk_name.md) | Underlying type contract |
