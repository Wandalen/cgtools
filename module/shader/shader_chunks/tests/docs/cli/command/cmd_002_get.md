# Command Test :: get

Source: [`../../../../docs/cli/command/02_get.md`](../../../../docs/cli/command/02_get.md)

### Parameter Edge Tests (PAR-N)

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | Valid, known chunk name (`hash21`) | `shader_chunks_test.rs::get_chunk_reports_full_detail_for_hash21`; `cli_subprocess_test.rs::get_hash21_prints_full_detail` |
| PAR-2 | Unknown chunk name | `shader_chunks_test.rs::get_chunk_reports_unknown_chunk_error_for_bogus_name`; `cli_subprocess_test.rs::get_unknown_chunk_exits_non_zero_without_a_panic_backtrace` |

Full edge-case detail: [`../param/01_name.md`](../param/01_name.md) EC-1/EC-2.

### Parameter Group Corner Tests (GRP-N)

*N/A — this CLI declares no parameter groups; see
[`../../../../docs/cli/readme.md` § Scope
Decisions](../../../../docs/cli/readme.md#scope-decisions).*

### Integration Tests (INT-N)

| ID | Scenario | Real Test |
|----|----------|-----------|
| INT-1 | Subprocess exit code is non-zero and stderr carries no panic backtrace on an unknown chunk | `cli_subprocess_test.rs::get_unknown_chunk_exits_non_zero_without_a_panic_backtrace` |

See also [`001_chunk.md`](001_chunk.md) INT-2/INT-3 for `get`'s role
following `list`/`tags` in a discover-then-inspect workflow.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 2 |
| GRP-N | 0 (no parameter groups) |
| INT-N | 1 |

### See Also

- [`../../../../docs/cli/command/02_get.md`](../../../../docs/cli/command/02_get.md) — command source
- [`../param/01_name.md`](../param/01_name.md) — `name` parameter
- [`../../../../docs/cli/format/03_plain_text.md`](../../../../docs/cli/format/03_plain_text.md) — output format
