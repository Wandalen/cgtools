# Parameter Group Test :: filtering

Source: [`../../../../docs/cli/param_group/01_filtering.md`](../../../../docs/cli/param_group/01_filtering.md)

### Group Cases (GRP-N)

| ID | Interaction | Real Test |
|----|-------------|-----------|
| GRP-1 | `case::` modifies `pattern::` — the same needle flips between matching and not as the switch toggles | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_pattern_matches_case_insensitively_by_default`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_pattern_with_case_switch_demands_exact_case` |
| GRP-2 | `case::` modifies `exports::` with the same switch | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_exports_filter_matches_signatures_with_case_switch` |
| GRP-3 | `tags_mode::` combines multiple `tag::` selectors — `any` unions, `all` intersects the same selector list | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_tags_mode_any_unions_and_all_intersects_selectors` |
| GRP-4 | `transitive::` widens `depends_on::` — same target chunk, direct vs. closure result sets | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_depends_on_selects_direct_dependents_and_transitive_widens` |
| GRP-5 | `roots::` and `leaves::` combine conjunctively with each other | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_roots_and_leaves_select_graph_extremes` |
| GRP-6 | Filters compose end-to-end through the CLI binding (`pattern::` + `format::` co-occurring) | `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` |
| GRP-7 | `case::` modifies `source::` with the same switch | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_source_filter_matches_wgsl_body_text_with_case_switch` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 7 |
| Real test functions | 7 |
| Modifier pairs covered | 3 (`case`→`pattern`/`exports`/`source`, `tags_mode`→`tag`, `transitive`→`depends_on`) |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../param/readme.md`](../param/readme.md) | Member parameters' own edge cases |
| [`../command_group/01_query.md`](../command_group/01_query.md) | The command group whose engine this group filters |
