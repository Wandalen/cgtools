# Command Group Test :: Parameters

Source: [`../../../../docs/cli/command_group/01_parameters.md`](../../../../docs/cli/command_group/01_parameters.md)

### Group Cases (CG-N)

| ID | Invariant | Evidence |
|----|-----------|----------|
| CG-1 | Every declared `//@ param:` line produces exactly one row — name, kind, type, range, source | `shader_chunks_params/tests/tunables_test.rs::tunables_of_chunk_lists_declared_and_inferred_parameters` |
| CG-2 | A chunk with zero declared parameters prints an explicit message, never blank output or a false error | `shader_chunks_params/tests/tunables_test.rs::tunables_zero_declared_params_reports_explicit_message_not_blank_or_error`; `cli_subprocess_test.rs::tunables_unannotated_real_chunk_prints_explicit_empty_message` |
| CG-3 | An unknown chunk name fails loudly with a non-zero exit, never a panic | `shader_chunks_params/tests/tunables_test.rs::tunables_unknown_chunk_reports_unknown_chunk_error`; `cli_subprocess_test.rs::tunables_bogus_chunk_exits_non_zero_without_a_panic_backtrace` |
| CG-4 | Range source is correctly attributed: a declared `range(min, max)` clause renders `Declared`; an absent one renders `Inferred` at the heuristic's resolved value | `shader_chunks_params/tests/tunables_test.rs::tunables_of_chunk_lists_declared_and_inferred_parameters` |
| CG-5 | The help screen renders this group with exactly its documented membership (`tunables`) | `cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility` |

### Membership Coverage

Confirms the group's Semantic Coherence Test
("[`01_parameters.md`](../../../../docs/cli/command_group/01_parameters.md#semantic-coherence-test)")
holds for every current member:

| Command | Answers | Confirmed |
|---------|---------|-----------|
| `.tunables` | What tunable parameters (and ranges) a chunk declares | ✅ |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 5 |
| Behaviorally tested | 5 |
| Structurally verified | 0 |
| Membership coverage | 1/1 commands |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_tunables.md`](../command/cmd_001_tunables.md) | Member command test spec |
| [`../../../../docs/cli/command_group/01_parameters.md`](../../../../docs/cli/command_group/01_parameters.md) | Group documentation source |
