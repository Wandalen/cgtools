# Type Test :: ChunkName

Source: [`../../../../docs/cli/type/01_chunk_name.md`](../../../../docs/cli/type/01_chunk_name.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Construction/parsing: valid known name string resolves to the matching chunk descriptor via `chunk_find`'s O(1) `shader_chunks_core::chunk_get` lookup | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_names_selects_in_given_order_and_allows_duplicates`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_shows_fbm3_dependency_chain_in_order`; `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order` |
| TC-2 | Invalid-input rejection: unknown name string never resolves, and never panics — always `CliError::UnknownChunk` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_unknown_name_reports_unknown_chunk_error`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_depends_on_unknown_chunk_fails_loudly`; `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::tree_chunk_reports_unknown_chunk_error_for_bogus_name`; `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_chunks_reports_unknown_chunk_error_for_bogus_name`; `shader_chunks_params/tests/tunables_test.rs::tunables_unknown_chunk_reports_unknown_chunk_error` |
| TC-3 | Round-trip: the resolved chunk's own rendered `name` field matches the input string that resolved it | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_get_defaults_renders_expanded_records_with_detail_fields` (asserts the record's `name` row given input `"hash21"`) |
| TC-4 | List membership: every `shader_chunks_core::CHUNKS` row's `name` is itself valid input to the query engine, `tree`, `tunables`, and `compose` | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_list_defaults_renders_every_chunk_as_plain_table`; `shader_chunks_params/tests/tunables_test.rs::tunables_zero_declared_params_reports_explicit_message_not_blank_or_error` |

There is no dedicated serialization step for `ChunkName` — it is a bare
`String`/`&str` throughout (`docs/cli/type/01_chunk_name.md`'s disclosed
simplification), so "serialization" collapses into TC-3's round-trip
check rather than a distinct case.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 4 |
| Real test functions referenced | 11 |
| Construction/parsing | TC-1, TC-4 |
| Invalid-input rejection | TC-2 |
| Round-trip | TC-3 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/01_name.md`](../param/01_name.md) | `name` parameter — `tree`/`tunables` usage context for this type |
| [`../param/02_names.md`](../param/02_names.md) | `names` parameter — list-of-this-type usage context |
| [`../param/08_depends_on.md`](../param/08_depends_on.md) | `depends_on` parameter — filter usage context |
