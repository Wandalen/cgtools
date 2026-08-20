# Parameter Test :: reverse

Source: [`../../../../docs/cli/param/22_reverse.md`](../../../../docs/cli/param/22_reverse.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `true` on a mid-chain chunk: shows its dependents chain in order (`hash21` → `value_noise` → `fbm3`) | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_reverse_on_a_chunk_shows_its_dependents_chain_in_order`; `cli_subprocess_test.rs::tree_hash21_reverse_shows_the_dependents_chain` |
| EC-2 | `true` with no `name`: forest of every leaf chunk (nothing it depends on), not every forward root | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_reverse_with_no_name_shows_forest_of_every_leaf_chunk` |
| EC-3 | `true` on a chunk with no real dependents: shows just that chunk, no children | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_reverse_on_a_leaf_with_no_dependents_shows_just_that_chunk` |
| EC-4 | `true` with an unknown chunk name: fails as loudly as the forward walk | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_reverse_reports_unknown_chunk_error_for_bogus_name` |

### Simple Co-Dependencies

`.tree`-only modifier; no interaction with `name` beyond selecting which
edge set (`depends_on` vs. its inverse) the walk from `name` follows. No
interaction with any `.list`/`.get` parameter — `reverse` does not exist
on those commands.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 4 |
| Real test functions | 5 |
| P1 (structural output) | EC-1, EC-2, EC-3 |
| P0 (exit-code-affecting) | EC-4 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/07_switch.md`](../type/07_switch.md) | Underlying boolean coercion contract |
| [`../command/cmd_004_tree.md`](../command/cmd_004_tree.md) | Owning command test spec (PAR-4) |
| [`../command_group/02_graph.md`](../command_group/02_graph.md) | Group-level invariant this direction flip must still satisfy |
