# Command Test :: tree

Source: [`../../../../docs/cli/command/04_tree.md`](../../../../docs/cli/command/04_tree.md)

### Parameter Edge Tests (PAR-N)

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | Valid, known chunk name (`fbm3`) — shows `fbm3 -> value_noise -> hash21` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_shows_fbm3_dependency_chain_in_order`; `cli_subprocess_test.rs::tree_fbm3_shows_the_dependency_chain` |
| PAR-2 | Absent (omitted entirely) — shows the full forest | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_with_no_name_shows_forest_of_every_root_chunk` |
| PAR-3 | Unknown chunk name | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_reports_unknown_chunk_error_for_bogus_name` |
| PAR-4 | `reverse::1` — forward vs. reverse walk direction, forest-with-no-name, leaf with no dependents, unknown name | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_reverse_on_a_chunk_shows_its_dependents_chain_in_order`, `::tree_reverse_with_no_name_shows_forest_of_every_leaf_chunk`, `::tree_reverse_on_a_leaf_with_no_dependents_shows_just_that_chunk`, `::tree_reverse_reports_unknown_chunk_error_for_bogus_name`; `cli_subprocess_test.rs::tree_hash21_reverse_shows_the_dependents_chain` |

Full edge-case detail: [`../param/01_name.md`](../param/01_name.md) EC-1/EC-2/EC-3;
[`../param/22_reverse.md`](../param/22_reverse.md) EC-1 onward.

### Parameter Group Corner Tests (GRP-N)

*N/A — `tree`'s two parameters, `name` and `reverse`, deliberately belong
to no [parameter group](../param_group/readme.md) (selection/direction,
not filtering), so no group applies to this command.*

### Integration Tests (INT-N)

*Omitted — `tree`'s only cross-command role (feeding into `compose`) is
covered by [`compose`](../../../../../shader_chunks_compose/tests/docs/cli/command_group/01_compose.md) WF-1.*

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 4 |
| GRP-N | 0 (`name`/`reverse` belong to no group) |
| INT-N | 0 (see shader_chunks_compose command_group/01_compose.md WF-1) |

### See Also

- [`../../../../docs/cli/command/04_tree.md`](../../../../docs/cli/command/04_tree.md) — command source
- [`../param/01_name.md`](../param/01_name.md) — `name` parameter
- [`../param/22_reverse.md`](../param/22_reverse.md) — `reverse` parameter
- [`../../../../docs/cli/format/02_tree_aligned.md`](../../../../docs/cli/format/02_tree_aligned.md) — output format
