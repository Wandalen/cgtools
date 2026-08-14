# Command Test :: tunables

Source: [`../../../../docs/cli/command/06_tunables.md`](../../../../docs/cli/command/06_tunables.md)

### Parameter Edge Tests (PAR-N)

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | Chunk (fixture) with 2 declared `//@ param:` lines — one with an explicit range, one relying on inference | `shader_chunks_test.rs::tunables_of_chunk_lists_declared_and_inferred_parameters` |
| PAR-2 | Real bundled chunk with zero declared parameters | `shader_chunks_test.rs::tunables_zero_declared_params_reports_explicit_message_not_blank_or_error`; `cli_subprocess_test.rs::tunables_unannotated_real_chunk_prints_explicit_empty_message` |
| PAR-3 | Unknown chunk name | `shader_chunks_test.rs::tunables_unknown_chunk_reports_unknown_chunk_error`; `cli_subprocess_test.rs::tunables_bogus_chunk_exits_non_zero_without_a_panic_backtrace` |

### Parameter Group Corner Tests (GRP-N)

*N/A — `tunables`'s sole parameter `name` deliberately belongs to no
[parameter group](../param_group/readme.md) (it is selection, not
filtering), so no group applies to this command.*

### Integration Tests (INT-N)

| ID | Case | Real Test |
|----|------|-----------|
| INT-1 | `sch tunables hash21` and `shader_chunks tunables hash21` produce byte-identical stdout/exit code | `cli_subprocess_test.rs::sch_alias_binary_produces_identical_output_to_shader_chunks` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 3 |
| GRP-N | 0 (`name` belongs to no group) |
| INT-N | 1 |

### See Also

- [`../../../../docs/cli/command/06_tunables.md`](../../../../docs/cli/command/06_tunables.md) — command source
- [`../param/01_name.md`](../param/01_name.md) — `name` parameter
- [`../../../../docs/cli/format/01_table_plain.md`](../../../../docs/cli/format/01_table_plain.md) — output format
