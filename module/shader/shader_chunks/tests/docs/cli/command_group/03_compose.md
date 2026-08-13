# Command Group Test :: Compose

Source: [`../../../../docs/cli/command_group/03_compose.md`](../../../../docs/cli/command_group/03_compose.md)

### Group Cases (CG-N)

| ID | Invariant | Evidence |
|----|-----------|----------|
| CG-1 | Output order is dependency order, regardless of input order | `shader_chunks_test.rs::compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order`; `cli_subprocess_test.rs::compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order` |
| CG-2 | Missing dependencies fail loudly with non-zero exit, never partial output | `shader_chunks_test.rs::compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted`; `cli_subprocess_test.rs::compose_missing_dependency_exits_non_zero_without_a_panic_backtrace` |
| CG-3 | Cyclic dependencies fail loudly, never hang or panic | `shader_chunks_test.rs::try_compose_wgsl_reports_cyclic_dependency_error_on_synthetic_fixture` |
| CG-4 | An unknown chunk name fails loudly, never a panic | `shader_chunks_test.rs::compose_chunks_reports_unknown_chunk_error_for_bogus_name` |
| CG-5 | The help screen renders this group with exactly its documented membership (`compose`) | `cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility` |

### Membership Coverage

Confirms the group's Semantic Coherence Test
("[`03_compose.md`](../../../../docs/cli/command_group/03_compose.md#semantic-coherence-test)")
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
| WF-1 | `tree <name>` then `compose <names...>` — preview the dependency order ([Graph](02_graph.md)), then compose using it | `cli_subprocess_test.rs::tree_fbm3_shows_the_dependency_chain` + `cli_subprocess_test.rs::compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 5 |
| Behaviorally tested | 5 |
| Structurally verified | 0 |
| Workflow compositions | 1 (cross-group, with Graph) |
| Membership coverage | 1/1 commands |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_005_compose.md`](../command/cmd_005_compose.md) | Member command test spec |
| [`../../../../docs/cli/command_group/03_compose.md`](../../../../docs/cli/command_group/03_compose.md) | Group documentation source |
