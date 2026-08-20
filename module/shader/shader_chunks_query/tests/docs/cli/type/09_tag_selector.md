# Type Test :: TagSelector

Source: [`../../../../docs/cli/type/09_tag_selector.md`](../../../../docs/cli/type/09_tag_selector.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Bare form (`tag`): matches the tag under any group | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_bare_tag_selector_matches_the_tag_under_any_group` |
| TC-2 | Pair form (`group:tag`, via `split_once(':')`): demands the exact group | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_pair_tag_selector_demands_the_exact_group` |
| TC-3 | Multiple selectors combine per `tags_mode::` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_tags_mode_any_unions_and_all_intersects_selectors` |
| TC-4 | End-to-end through the CLI binding | `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` (`tag::noise`) |

`TagSelector` is an open selector: there is no invalid form — any string
parses (pair if it contains `:`, bare otherwise), and an unmatched
selector yields empty output with exit 0, never an error.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 4 |
| Real test functions referenced | 4 |
| Bare-form parsing | TC-1 |
| Pair-form parsing | TC-2 |
| Combination semantics | TC-3 |
| End-to-end binding | TC-4 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/05_tag.md`](../param/05_tag.md) | `tag` parameter — the sole usage context |
| [`../type/06_tags_mode.md`](06_tags_mode.md) | The mode combining multiple selectors |
