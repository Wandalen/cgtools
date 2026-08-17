# Parameter Test :: shape

Source: [`../../../../docs/cli/param/24_shape.md`](../../../../docs/cli/param/24_shape.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | All three values round-trip through `parse`/`as_str` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_format_round_trips_and_rejects_bogus_values` |
| EC-2 | Bogus value rejected as `InvalidParam { param : "shape" }`, exit 1 | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_format_round_trips_and_rejects_bogus_values` |
| EC-3 | `dot` renders one `"parent" -> "child";` edge per dependency, in walk order | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_dot_format_renders_digraph_with_edges_in_dependency_order` |
| EC-4 | `mermaid` renders one `parent --> child` edge per dependency, in walk order | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_mermaid_format_renders_graph_td_with_edges_in_dependency_order` |
| EC-5 | `dot` on a childless root declares a bare node, no edges | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_dot_format_declares_childless_root_with_no_edges` |
| EC-6 | `mermaid` on a childless root declares a bare node, no edges | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_mermaid_format_declares_childless_root_with_no_edges` |
| EC-7 | `reverse::1` flips `dot`/`mermaid` edge direction identically to `aligned` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_dot_and_mermaid_reverse_walk_flip_edge_direction_like_aligned` |
| EC-8 | `name` omitted — `dot`/`mermaid` combine every root's edges (or bare declaration) into one graph, not separate outputs | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_dot_and_mermaid_with_no_name_combine_every_root_into_one_graph` |

### Simple Co-Dependencies

`.tree`-only modifier, same as [`reverse`](22_reverse.md). Interacts with
`reverse::` ( selects which edge set — `depends_on` or its inverse — the
walk `shape::` then renders ) but not with `name` beyond the usual root
selection.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 8 |
| Real test functions | 7 |
| P0 (exit-code-affecting) | EC-2 |
| P1 (structural output) | EC-1, EC-3, EC-4, EC-5, EC-6, EC-7, EC-8 |

### Cross-References

| File | Relationship |
|------|------|
| [`../type/11_tree_format.md`](../type/11_tree_format.md) | Underlying enum contract |
| [`../../../../docs/cli/format/09_tree_dot.md`](../../../../docs/cli/format/09_tree_dot.md) | `dot` structural spec |
| [`../../../../docs/cli/format/10_tree_mermaid.md`](../../../../docs/cli/format/10_tree_mermaid.md) | `mermaid` structural spec |
