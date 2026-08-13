# Parameter Test :: pattern

Source: [`../../../../docs/cli/param/03_pattern.md`](../../../../docs/cli/param/03_pattern.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Matching needle, mixed case, default insensitivity | `shader_chunks_test.rs::query_pattern_matches_case_insensitively_by_default` |
| EC-2 | Same needle under `case::1` — no exact-case match, empty result, exit 0 | `shader_chunks_test.rs::query_pattern_with_case_switch_demands_exact_case` |
| EC-3 | End-to-end through the CLI binding | `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` |

### Simple Co-Dependencies

Modified by [`case::`](04_case.md); combines conjunctively with every
other [filtering](../param_group/01_filtering.md) member.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 3 |
| Real test functions | 3 |
| P1 (structural output) | EC-1, EC-2, EC-3 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../param_group/01_filtering.md`](../param_group/01_filtering.md) | Interaction with `case::` |
| [`../command/cmd_001_list.md`](../command/cmd_001_list.md) | Primary consuming command |
