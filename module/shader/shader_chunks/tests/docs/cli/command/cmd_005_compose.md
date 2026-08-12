# Command Test :: compose

Source: [`../../../../docs/cli/command/05_compose.md`](../../../../docs/cli/command/05_compose.md)

### Parameter Edge Tests (PAR-N)

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | 2 valid names, either input order — dependency-ordered output | `shader_chunks_test.rs::compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order`; `cli_subprocess_test.rs::compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order` |
| PAR-2 | Unknown chunk name among the list | `shader_chunks_test.rs::compose_chunks_reports_unknown_chunk_error_for_bogus_name` |
| PAR-3 | Valid names, missing a declared dependency | `shader_chunks_test.rs::compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted`; `cli_subprocess_test.rs::compose_missing_dependency_exits_non_zero_without_a_panic_backtrace` |
| PAR-4 | Cyclic dependency (synthetic fixture) | `shader_chunks_test.rs::try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture` |

Full edge-case detail: [`../param/02_names.md`](../param/02_names.md) EC-1 through EC-5.

### Parameter Group Corner Tests (GRP-N)

*N/A — this CLI declares no parameter groups; see
[`../../../../docs/cli/readme.md` § Scope
Decisions](../../../../docs/cli/readme.md#scope-decisions).*

### Integration Tests (INT-N)

See also [`001_chunk.md`](001_chunk.md) INT-4 for `compose`'s role
following `tree` in a preview-then-compose workflow.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 4 |
| GRP-N | 0 (no parameter groups) |
| INT-N | 0 (see 001_chunk.md INT-4) |

### See Also

- [`../../../../docs/cli/command/05_compose.md`](../../../../docs/cli/command/05_compose.md) — command source
- [`../param/02_names.md`](../param/02_names.md) — `names` parameter
- [`../../../../docs/cli/format/03_plain_text.md`](../../../../docs/cli/format/03_plain_text.md) — output format
