# Type Test :: Switch

Source: [`../../../../docs/cli/type/07_switch.md`](../../../../docs/cli/type/07_switch.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Off (default `false`): filter/modifier behavior absent | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_pattern_matches_case_insensitively_by_default` (`case`); `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_depends_on_selects_direct_dependents_and_transitive_widens` (`transitive` off arm); `shader_chunks_render/tests/render_cli_test.rs::name_target_renders_a_png_of_the_requested_size` (`all` off — renders exactly one target, unaffected by batch machinery) |
| TC-2 | On (`::1` → `true`): behavior engages for each of the 5 query switch params plus `.tree`'s `reverse` and `.render`'s `all` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_pattern_with_case_switch_demands_exact_case` (`case`); `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_depends_on_selects_direct_dependents_and_transitive_widens` (`transitive`); `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_roots_and_leaves_select_graph_extremes` (`roots`, `leaves`); `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_count_reports_filtered_total_before_paging` (`count`); `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_reverse_on_a_chunk_shows_its_dependents_chain_in_order` (`reverse`); `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_all_writes_a_png_per_chunk_into_a_freshly_created_dir_and_reports_totals` (`all`) |
| TC-3 | End-to-end coercion through unilang's `Kind::Boolean` binding | `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` (`roots::1`, `count::1`) |
| TC-4 | `serve` (only inverted-default switch, `true`): `serve::0` exercised directly (skips the browser hand-off, still writes/validates the bundle); the `true`/default hand-off path is not exercised by any test — it would block the test process on a live browser dev server | `shader_chunks_preview/tests/preview_cli_test.rs::preview_without_serve_writes_the_bundle_into_the_web_runner_crate`; `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_serve_0_succeeds_and_prints_the_summary` |
| TC-5 | Invalid-token rejection through the boolean kind: `serve::maybe` exits non-zero before the routine runs (timeout-guarded so a silently-accepted value blocking on the browser server fails loudly) | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_bad_serve_value_is_rejected_by_coercion` |

Parsing itself is delegated to unilang's boolean kind — the crate never
hand-parses switch strings, so there is no crate-level parser to
unit-test; TC-3 pins the end-to-end acceptance binding and TC-5 the
rejection side of the same coercion. TC-4 is kept separate from
TC-1/TC-2 rather than folded into "the 6th switch param" because
`serve` inverts the off/default relationship those two rows describe
(see [`serve`](../../../../../shader_chunks_preview/tests/docs/cli/param/02_serve.md)'s EC table for the
parameter-tier detail).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 5 |
| Real test functions referenced | 12 |
| Off state | TC-1 |
| On state | TC-2 (7 of 8 switch params exercised via explicit `::1`) |
| End-to-end binding | TC-3 |
| Inverted-default state | TC-4 (`serve`; `serve::0` demonstrated, default `true` path undemonstrated — disclosed gap) |
| Invalid-input rejection | TC-5 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/04_case.md`](../param/04_case.md) | Switch param: matcher case sensitivity |
| [`../param/09_transitive.md`](../param/09_transitive.md) | Switch param: closure widening |
| [`../param/11_roots.md`](../param/11_roots.md) | Switch param: root filter |
| [`../param/12_leaves.md`](../param/12_leaves.md) | Switch param: leaf filter |
| [`../param/14_count.md`](../param/14_count.md) | Switch param: count short-circuit |
| [`serve`](../../../../../shader_chunks_preview/tests/docs/cli/param/02_serve.md) | Switch param: browser hand-off toggle (inverted default) |
| [`../param/22_reverse.md`](../param/22_reverse.md) | Switch param: `.tree` walk-direction flip |
| [`all`](../../../../../shader_chunks_render/tests/docs/cli/param/05_all.md) | Switch param: `.render` batch-sweep toggle |
