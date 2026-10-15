# Command Test :: tree

Source: [`../../../../docs/cli/command/04_tree.md`](../../../../docs/cli/command/04_tree.md)

### Parameter Edge Tests (PAR-N)

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | Valid, known chunk name (`fbm3`) — shows `fbm3 -> value_noise -> hash21` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_shows_fbm3_dependency_chain_in_order`; `cli_subprocess_test.rs::tree_fbm3_shows_the_dependency_chain` |
| PAR-2 | Absent (omitted entirely) — shows the full forest | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_with_no_name_shows_forest_of_every_root_chunk` |
| PAR-3 | Unknown chunk name | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_reports_unknown_chunk_error_for_bogus_name` |
| PAR-4 | `reverse::1` — forward vs. reverse walk direction, forest-with-no-name, leaf with no dependents, unknown name | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_reverse_on_a_chunk_shows_its_dependents_chain_in_order`, `::tree_reverse_with_no_name_shows_forest_of_every_leaf_chunk`, `::tree_reverse_on_a_leaf_with_no_dependents_shows_just_that_chunk`, `::tree_reverse_reports_unknown_chunk_error_for_bogus_name`; `cli_subprocess_test.rs::tree_hash21_reverse_shows_the_dependents_chain` |
| PAR-5 | `shape::dot`/`shape::mermaid` — structural rendering, childless-root declaration, no-name forest combination, reverse-walk edge flip, bogus value rejection | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_dot_format_renders_digraph_with_edges_in_dependency_order`, `::tree_dot_format_declares_childless_root_with_no_edges`, `::tree_mermaid_format_renders_graph_td_with_edges_in_dependency_order`, `::tree_mermaid_format_declares_childless_root_with_no_edges`, `::tree_dot_and_mermaid_with_no_name_combine_every_root_into_one_graph`, `::tree_dot_and_mermaid_reverse_walk_flip_edge_direction_like_aligned`, `::tree_format_round_trips_and_rejects_bogus_values`; `cli_subprocess_test.rs::tree_fbm3_shape_dot_and_mermaid_render_the_same_chain_as_edges` |

Full edge-case detail: [`../param/01_name.md`](../param/01_name.md) EC-1/EC-2/EC-3;
[`../param/22_reverse.md`](../param/22_reverse.md) EC-1 onward;
[`../param/24_shape.md`](../param/24_shape.md) EC-1 onward.

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
| PAR-N | 5 |
| GRP-N | 0 (`name`/`reverse`/`shape` belong to no group) |
| INT-N | 0 (see shader_chunks_compose command_group/01_compose.md WF-1) |

### See Also

- [`../../../../docs/cli/command/04_tree.md`](../../../../docs/cli/command/04_tree.md) — command source
- [`../param/01_name.md`](../param/01_name.md) — `name` parameter
- [`../param/22_reverse.md`](../param/22_reverse.md) — `reverse` parameter
- [`../param/24_shape.md`](../param/24_shape.md) — `shape` parameter
- [`../../../../docs/cli/format/02_tree_aligned.md`](../../../../docs/cli/format/02_tree_aligned.md) — aligned output format
- [`../../../../docs/cli/format/09_tree_dot.md`](../../../../docs/cli/format/09_tree_dot.md) — dot output format
- [`../../../../docs/cli/format/10_tree_mermaid.md`](../../../../docs/cli/format/10_tree_mermaid.md) — mermaid output format
