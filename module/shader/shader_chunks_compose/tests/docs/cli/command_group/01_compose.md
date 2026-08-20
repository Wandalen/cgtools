# Command Group Test :: Compose

Source: [`../../../../docs/cli/command_group/01_compose.md`](../../../../docs/cli/command_group/01_compose.md)

### Group Cases (CG-N)

| ID | Invariant | Evidence |
|----|-----------|----------|
| CG-1 | Output order is dependency order, regardless of input order | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order`; `cli_subprocess_test.rs::compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order` |
| CG-2 | Missing dependencies fail loudly with non-zero exit, never partial output | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted`; `cli_subprocess_test.rs::compose_missing_dependency_exits_non_zero_without_a_panic_backtrace` |
| CG-3 | Cyclic dependencies fail loudly, never hang or panic | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture` |
| CG-4 | An unknown chunk name fails loudly, never a panic | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_chunks_reports_unknown_chunk_error_for_bogus_name` |
| CG-5 | The help screen renders this group with exactly its documented membership (`compose`) | `cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility` |
| CG-6 | With `out::` given, the composed WGSL is written to the file and stdout carries only the summary; without it, stdout still carries the composed text as before | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::subprocess_compose_writes_the_file_and_prints_the_summary`; `subprocess_compose_without_out_prints_composed_text_to_stdout` |
| CG-7 | A write-side failure at `out::` (missing parent directory) is a distinct `Io` error, exit 2, and never leaves a partial file | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_write_to_an_unwritable_path_is_an_io_error_with_exit_code_2`; `subprocess_compose_out_to_unwritable_path_fails_with_exit_2` |

### Membership Coverage

Confirms the group's Semantic Coherence Test
("[`01_compose.md`](../../../../docs/cli/command_group/01_compose.md#semantic-coherence-test)")
holds for every current member:

| Command | Answers | Confirmed |
|---------|---------|-----------|
| `.compose` | What WGSL text would composing these chunks produce | ✅ |

### Workflow Compositions

The cross-group preview-then-compose workflow, verified by composing
each step's own independently-passing test (both commands are stateless
and idempotent, so no dedicated multi-invocation test is needed):

| ID | Workflow | Composed From (Real Tests) |
|----|----------|------------------------------|
| WF-1 | `tree <name>` then `compose <names...>` — preview the dependency order ([Graph](../../../../../shader_chunks_query/tests/docs/cli/command_group/02_graph.md)), then compose using it | `cli_subprocess_test.rs::tree_fbm3_shows_the_dependency_chain` + `cli_subprocess_test.rs::compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 7 |
| Behaviorally tested | 7 |
| Structurally verified | 0 |
| Workflow compositions | 1 (cross-group, with Graph) |
| Membership coverage | 1/1 commands |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_compose.md`](../command/cmd_001_compose.md) | Member command test spec |
| [`../../../../docs/cli/command_group/01_compose.md`](../../../../docs/cli/command_group/01_compose.md) | Group documentation source |
