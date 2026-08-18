# Command Test :: compose

Source: [`../../../../docs/cli/command/01_compose.md`](../../../../docs/cli/command/01_compose.md)

### Parameter Edge Tests (PAR-N)

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | 2 valid names, either input order — dependency-ordered output | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order`; `cli_subprocess_test.rs::compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order` |
| PAR-2 | Unknown chunk name among the list | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_chunks_reports_unknown_chunk_error_for_bogus_name` |
| PAR-3 | Valid names, missing a declared dependency | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted`; `cli_subprocess_test.rs::compose_missing_dependency_exits_non_zero_without_a_panic_backtrace` |
| PAR-4 | Cyclic dependency (synthetic fixture) | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture` |
| PAR-5 | `transitive::1` widens one root name to its full dependency closure, identical to the explicit set; strict default unchanged | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_chunks_transitive_closure_equals_the_explicit_full_set`; `cli_subprocess_test.rs::compose_single_name_with_transitive_pulls_the_full_dependency_chain` |
| PAR-6 | `compose_write` writes the composed text verbatim and returns a `wrote <path> (<n> bytes wgsl)` summary | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_write_writes_the_composed_text_and_returns_a_byte_count_summary` |
| PAR-7 | `compose_write` to an unwritable path (missing parent directory) — `ComposeCliError::Io`, exit 2, no file left behind | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_write_to_an_unwritable_path_is_an_io_error_with_exit_code_2` |

Full edge-case detail: [`names`](../../../../../shader_chunks_query/tests/docs/cli/param/02_names.md) EC-4
through EC-7; [`transitive`](../../../../../shader_chunks_query/tests/docs/cli/param/09_transitive.md) EC-3
through EC-5 for the closure semantics; [`out`](../param/01_out.md) EC-1
through EC-6 for the file-output semantics.

### Parameter Group Corner Tests (GRP-N)

*N/A — `compose`'s positional `names` deliberately belongs to no
[parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md)
(it is selection, not filtering), and `transitive::` is the only
[filtering](../../../../../shader_chunks_query/tests/docs/cli/param_group/01_filtering.md)-group member `compose`
accepts, so no within-group combination exists to corner-test.*

### Integration Tests (INT-N)

Subprocess-level, end-to-end cases for `out::`; see also
[`../command_group/01_compose.md`](../command_group/01_compose.md) WF-1
for `compose`'s role following `tree` in a preview-then-compose
workflow.

| ID | Scenario | Real Test |
|----|----------|-----------|
| INT-1 | `compose <names...> out::<tmp>` writes the file and prints only the summary (never the WGSL) to stdout | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::subprocess_compose_writes_the_file_and_prints_the_summary` |
| INT-2 | `compose <names...>` without `out::` — composed text still goes to stdout, no summary line appears | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::subprocess_compose_without_out_prints_composed_text_to_stdout` |
| INT-3 | `compose fbm3 transitive::1 out::<tmp>` — the file receives the full dependency closure | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::subprocess_compose_with_out_and_transitive_writes_the_full_closure` |
| INT-4 | `compose <names...> out::<unwritable>` — exit 2, `io error` in stderr, no file left behind | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::subprocess_compose_out_to_unwritable_path_fails_with_exit_2` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 7 |
| GRP-N | 0 (no within-group combination available) |
| INT-N | 4 |

### See Also

- [`../../../../docs/cli/command/01_compose.md`](../../../../docs/cli/command/01_compose.md) — command source
- [`names`](../../../../../shader_chunks_query/tests/docs/cli/param/02_names.md) — `names` parameter
- [`transitive`](../../../../../shader_chunks_query/tests/docs/cli/param/09_transitive.md) — `transitive` parameter
- [`../param/01_out.md`](../param/01_out.md) — `out` parameter
- [`../../../../docs/cli/format/01_plain_text.md`](../../../../docs/cli/format/01_plain_text.md) — output format
