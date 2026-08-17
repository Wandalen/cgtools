# Type Test :: TreeFormat

Source: [`../../../../docs/cli/type/11_tree_format.md`](../../../../docs/cli/type/11_tree_format.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Round-trip: all 3 variants survive `as_str` → `parse` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_format_round_trips_and_rejects_bogus_values` |
| TC-2 | Invalid-input rejection: bogus string is `QueryError::InvalidParam { param : "shape" }`, exit 1 | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_format_round_trips_and_rejects_bogus_values`; `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |
| TC-3 | Behavioral: each parsed variant drives its distinct renderer, including the multi-root forest case | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_dot_format_renders_digraph_with_edges_in_dependency_order`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_mermaid_format_renders_graph_td_with_edges_in_dependency_order`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_dot_and_mermaid_with_no_name_combine_every_root_into_one_graph`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_shows_fbm3_dependency_chain_in_order` (`Aligned`); `cli_subprocess_test.rs::tree_fbm3_shape_dot_and_mermaid_render_the_same_chain_as_edges` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 3 |
| Real test functions referenced | 6 |
| Round-trip | TC-1 |
| Invalid-input rejection | TC-2 |
| Renderer dispatch | TC-3 (all 3 variants exercised) |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/24_shape.md`](../param/24_shape.md) | `shape` parameter — the sole usage context |
| [`../../../../docs/cli/format/09_tree_dot.md`](../../../../docs/cli/format/09_tree_dot.md) | `dot` structural spec |
| [`../../../../docs/cli/format/10_tree_mermaid.md`](../../../../docs/cli/format/10_tree_mermaid.md) | `mermaid` structural spec |
