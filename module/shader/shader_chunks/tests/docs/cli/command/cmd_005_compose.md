# Command Test :: compose

Source: [`../../../../docs/cli/command/05_compose.md`](../../../../docs/cli/command/05_compose.md)

### Parameter Edge Tests (PAR-N)

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | 2 valid names, either input order — dependency-ordered output | `shader_chunks_test.rs::compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order`; `cli_subprocess_test.rs::compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order` |
| PAR-2 | Unknown chunk name among the list | `shader_chunks_test.rs::compose_chunks_reports_unknown_chunk_error_for_bogus_name` |
| PAR-3 | Valid names, missing a declared dependency | `shader_chunks_test.rs::compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted`; `cli_subprocess_test.rs::compose_missing_dependency_exits_non_zero_without_a_panic_backtrace` |
| PAR-4 | Cyclic dependency (synthetic fixture) | `shader_chunks_test.rs::try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture` |
| PAR-5 | `transitive::1` widens one root name to its full dependency closure, identical to the explicit set; strict default unchanged | `shader_chunks_test.rs::compose_chunks_transitive_closure_equals_the_explicit_full_set`; `cli_subprocess_test.rs::compose_single_name_with_transitive_pulls_the_full_dependency_chain` |

Full edge-case detail: [`../param/02_names.md`](../param/02_names.md) EC-4
through EC-7; [`../param/09_transitive.md`](../param/09_transitive.md) EC-3
through EC-5 for the closure semantics.

### Parameter Group Corner Tests (GRP-N)

*N/A — `compose`'s positional `names` deliberately belongs to no
[parameter group](../param_group/readme.md) (it is selection, not
filtering), and `transitive::` is the only
[filtering](../param_group/01_filtering.md)-group member `compose`
accepts, so no within-group combination exists to corner-test.*

### Integration Tests (INT-N)

See also [`../command_group/03_compose.md`](../command_group/03_compose.md)
WF-1 for `compose`'s role following `tree` in a preview-then-compose
workflow.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 5 |
| GRP-N | 0 (no within-group combination available) |
| INT-N | 0 (see command_group/03_compose.md WF-1) |

### See Also

- [`../../../../docs/cli/command/05_compose.md`](../../../../docs/cli/command/05_compose.md) — command source
- [`../param/02_names.md`](../param/02_names.md) — `names` parameter
- [`../param/09_transitive.md`](../param/09_transitive.md) — `transitive` parameter
- [`../../../../docs/cli/format/03_plain_text.md`](../../../../docs/cli/format/03_plain_text.md) — output format
