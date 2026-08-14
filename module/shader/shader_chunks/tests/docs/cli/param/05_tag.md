# Parameter Test :: tag

Source: [`../../../../docs/cli/param/05_tag.md`](../../../../docs/cli/param/05_tag.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Bare selector matches the tag part under any group | `shader_chunks_test.rs::query_bare_tag_selector_matches_the_tag_under_any_group` |
| EC-2 | Pair selector demands the exact `group:tag`; a wrong group matches nothing | `shader_chunks_test.rs::query_pair_tag_selector_demands_the_exact_group` |
| EC-3 | Comma-separated multi-selector list combines per `tags_mode::` | `shader_chunks_test.rs::query_tags_mode_any_unions_and_all_intersects_selectors` |
| EC-4 | End-to-end through the CLI binding (`tag::noise` + format) | `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` |

### Simple Co-Dependencies

Multi-selector combination is governed by
[`tags_mode::`](06_tags_mode.md); the selector vocabulary is enumerable
via the `tags` command.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 4 |
| Real test functions | 4 |
| P1 (structural output) | EC-1 … EC-4 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/09_tag_selector.md`](../type/09_tag_selector.md) | Selector parsing contract |
| [`../param_group/01_filtering.md`](../param_group/01_filtering.md) | Interaction with `tags_mode::` |
