# Type Test :: Switch

Source: [`../../../../docs/cli/type/07_switch.md`](../../../../docs/cli/type/07_switch.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Off (default `false`): filter/modifier behavior absent | `shader_chunks_test.rs::query_pattern_matches_case_insensitively_by_default` (`case`); `shader_chunks_test.rs::query_depends_on_selects_direct_dependents_and_transitive_widens` (`transitive` off arm) |
| TC-2 | On (`::1` → `true`): behavior engages for each of the 5 switch params | `shader_chunks_test.rs::query_pattern_with_case_switch_demands_exact_case` (`case`); `shader_chunks_test.rs::query_depends_on_selects_direct_dependents_and_transitive_widens` (`transitive`); `shader_chunks_test.rs::query_roots_and_leaves_select_graph_extremes` (`roots`, `leaves`); `shader_chunks_test.rs::query_count_reports_filtered_total_before_paging` (`count`) |
| TC-3 | End-to-end coercion through unilang's `Kind::Boolean` binding | `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` (`roots::1`, `count::1`) |

Parsing itself is delegated to unilang's boolean kind — the crate never
hand-parses switch strings, so there is no crate-level rejection case to
pin; TC-3 pins the end-to-end binding instead.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 3 |
| Real test functions referenced | 6 |
| Off state | TC-1 |
| On state | TC-2 (all 5 switch params exercised) |
| End-to-end binding | TC-3 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/04_case.md`](../param/04_case.md) | Switch param: matcher case sensitivity |
| [`../param/09_transitive.md`](../param/09_transitive.md) | Switch param: closure widening |
| [`../param/11_roots.md`](../param/11_roots.md) | Switch param: root filter |
| [`../param/12_leaves.md`](../param/12_leaves.md) | Switch param: leaf filter |
| [`../param/14_count.md`](../param/14_count.md) | Switch param: count short-circuit |
