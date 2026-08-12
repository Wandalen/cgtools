# Command Test :: list

Source: [`../../../../docs/cli/command/01_list.md`](../../../../docs/cli/command/01_list.md)

### Parameter Edge Tests (PAR-N)

*N/A — `.list` declares zero parameters (see
[`../../../../docs/cli/command/01_list.md`](../../../../docs/cli/command/01_list.md)'s
Parameters table).*

### Parameter Group Corner Tests (GRP-N)

*N/A — this CLI declares no parameter groups; see
[`../../../../docs/cli/readme.md` § Scope
Decisions](../../../../docs/cli/readme.md#scope-decisions).*

### Integration Tests (INT-N)

| ID | Scenario | Real Test |
|----|----------|-----------|
| INT-1 | Direct call lists all 4 bundled chunks with expected columns | `shader_chunks_cli_test.rs::list_chunks_lists_all_four_bundled_chunks_with_expected_columns` |
| INT-2 | Subprocess invocation prints a table with all 4 bundled chunks | `cli_subprocess_test.rs::list_prints_a_table_with_all_four_bundled_chunks` |

See also [`001_chunk.md`](001_chunk.md) INT-2/INT-3 for `list`'s role in a
larger discover-then-inspect workflow.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 0 (no parameters) |
| GRP-N | 0 (no parameter groups) |
| INT-N | 2 |

### See Also

- [`../../../../docs/cli/command/01_list.md`](../../../../docs/cli/command/01_list.md) — command source
- [`../../../../docs/cli/format/01_table_plain.md`](../../../../docs/cli/format/01_table_plain.md) — output format
