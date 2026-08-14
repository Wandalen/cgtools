# Command Group Test :: Graph

Source: [`../../../../docs/cli/command_group/02_graph.md`](../../../../docs/cli/command_group/02_graph.md)

### Group Cases (CG-N)

| ID | Invariant | Evidence |
|----|-----------|----------|
| CG-1 | Nesting mirrors declared `depends_on` metadata exactly — parent before child, chain order preserved | `shader_chunks_test.rs::tree_chunk_shows_fbm3_dependency_chain_in_order`; `cli_subprocess_test.rs::tree_fbm3_shows_the_dependency_chain` |
| CG-2 | The no-argument forest renders exactly the root chunks | `shader_chunks_test.rs::tree_chunk_with_no_name_shows_forest_of_every_root_chunk` |
| CG-3 | An unknown root name fails loudly, never a panic | `shader_chunks_test.rs::tree_chunk_reports_unknown_chunk_error_for_bogus_name` |
| CG-4 | The help screen renders this group with exactly its documented membership (`tree`) | `cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility` |

### Membership Coverage

Confirms the group's Semantic Coherence Test
("[`02_graph.md`](../../../../docs/cli/command_group/02_graph.md#semantic-coherence-test)")
holds for every current member:

| Command | Answers | Confirmed |
|---------|---------|-----------|
| `.tree` | What depends on what, rendered as the graph itself | ✅ |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 4 |
| Behaviorally tested | 4 |
| Structurally verified | 0 |
| Membership coverage | 1/1 commands |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_004_tree.md`](../command/cmd_004_tree.md) | Member command test spec |
| [`../../../../docs/cli/command_group/02_graph.md`](../../../../docs/cli/command_group/02_graph.md) | Group documentation source |
