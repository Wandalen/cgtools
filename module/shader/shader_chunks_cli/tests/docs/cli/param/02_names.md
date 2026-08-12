# Parameter Test :: names

Source: [`../../../../docs/cli/param/02_names.md`](../../../../docs/cli/param/02_names.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | 2 valid names, dependency-first input order | `shader_chunks_cli_test.rs::compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order`; `cli_subprocess_test.rs::compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order` |
| EC-2 | 2 valid names, reverse input order (order-independence) | `shader_chunks_cli_test.rs::compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order` (same test asserts both orderings) |
| EC-3 | Unknown chunk name among the list | `shader_chunks_cli_test.rs::compose_chunks_reports_unknown_chunk_error_for_bogus_name` |
| EC-4 | Valid names, but a declared dependency omitted from the list | `shader_chunks_cli_test.rs::compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted`; `cli_subprocess_test.rs::compose_missing_dependency_exits_non_zero_without_a_panic_backtrace` |
| EC-5 | Cyclic dependency (synthetic fixture, not a real bundled chunk) | `shader_chunks_cli_test.rs::try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture` |

### Simple Co-Dependencies

`names` is the sole parameter `compose` declares — no other parameter
interacts with it. Its list cardinality (≥1) is enforced by `unilang`'s
`ArgumentAttributes{ multiple: true }` before `CliError` handling is ever
reached.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 5 |
| Real test functions | 5 (1 shared across EC-1/EC-2) |
| P0 (exit-code-affecting) | EC-3, EC-4, EC-5 |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_005_compose.md`](../command/cmd_005_compose.md) | Command using `names` |
| [`../type/01_chunk_name.md`](../type/01_chunk_name.md) | Underlying type contract (applied per list element) |
