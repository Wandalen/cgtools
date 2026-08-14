# Parameter Test :: names

Source: [`../../../../docs/cli/param/02_names.md`](../../../../docs/cli/param/02_names.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Query selection keeps the given order and allows duplicates (`list`/`get`) | `shader_chunks_test.rs::query_names_selects_in_given_order_and_allows_duplicates` |
| EC-2 | Unknown chunk name in a query selection fails loudly | `shader_chunks_test.rs::query_unknown_name_reports_unknown_chunk_error`; `cli_subprocess_test.rs::get_unknown_chunk_exits_non_zero_without_a_panic_backtrace` |
| EC-3 | Absent: `list` selects every chunk, `get` fails loudly ("required argument 'names' is missing") | `shader_chunks_test.rs::query_list_defaults_renders_all_four_chunks_as_plain_table`; `cli_subprocess_test.rs::get_without_names_fails_loudly_while_list_succeeds` |
| EC-4 | 2 valid names for `compose`, either input order (order-independence) | `shader_chunks_test.rs::compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order`; `cli_subprocess_test.rs::compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order` |
| EC-5 | Unknown chunk name among `compose`'s list | `shader_chunks_test.rs::compose_chunks_reports_unknown_chunk_error_for_bogus_name` |
| EC-6 | Valid names, but a declared dependency omitted from `compose`'s list | `shader_chunks_test.rs::compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted`; `cli_subprocess_test.rs::compose_missing_dependency_exits_non_zero_without_a_panic_backtrace` |
| EC-7 | Cyclic dependency (synthetic fixture, not a real bundled chunk) | `shader_chunks_test.rs::try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture` |

### Simple Co-Dependencies

For `list`/`get`, `names` fixes the candidate set the
[filtering](../param_group/01_filtering.md) group then narrows, and
`sort::input` preserves its order — see
`shader_chunks_test.rs::query_sort_keys_order_deterministically`. For
`compose` it is the only positional parameter, joined by the named
[`transitive::`](09_transitive.md) closure switch. Requiredness differs per command
(EC-3): optional on `list`, required (≥1) on `get`/`compose`, enforced by
`unilang` before `CliError` handling is ever reached.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 7 |
| Real test functions | 10 |
| P0 (exit-code-affecting) | EC-2, EC-3 (get half), EC-5, EC-6, EC-7 |
| P1 (structural output) | EC-1, EC-3 (list half), EC-4 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_list.md`](../command/cmd_001_list.md) | Command using `names` as optional selection |
| [`../command/cmd_002_get.md`](../command/cmd_002_get.md) | Command using `names` as required selection |
| [`../command/cmd_005_compose.md`](../command/cmd_005_compose.md) | Command using `names` as required compose set |
| [`../type/01_chunk_name.md`](../type/01_chunk_name.md) | Underlying type contract (applied per list element) |
