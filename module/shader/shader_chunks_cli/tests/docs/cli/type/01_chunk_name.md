# Type Test :: ChunkName

Source: [`../../../../docs/cli/type/01_chunk_name.md`](../../../../docs/cli/type/01_chunk_name.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Construction/parsing: valid known name string resolves to the matching chunk body via `find_chunk`'s linear scan + `shader_chunks::parse_name` | `shader_chunks_cli_test.rs::get_chunk_reports_full_detail_for_hash21`; `shader_chunks_cli_test.rs::tree_chunk_shows_fbm3_dependency_chain_in_order`; `shader_chunks_cli_test.rs::compose_chunks_orders_hash21_before_value_noise_regardless_of_input_order` |
| TC-2 | Invalid-input rejection: unknown name string never resolves, and never panics — always `CliError::UnknownChunk` | `shader_chunks_cli_test.rs::get_chunk_reports_unknown_chunk_error_for_bogus_name`; `shader_chunks_cli_test.rs::tree_chunk_reports_unknown_chunk_error_for_bogus_name`; `shader_chunks_cli_test.rs::compose_chunks_reports_unknown_chunk_error_for_bogus_name` |
| TC-3 | Round-trip: the resolved chunk's own parsed `name:` field matches the input string that resolved it | `shader_chunks_cli_test.rs::get_chunk_reports_full_detail_for_hash21` (asserts output's `name: hash21` line given input `"hash21"`) |
| TC-4 | List membership: every name produced by parsing `shader_chunks::ALL_CHUNKS` is itself valid input to `get`/`tree`/`compose` | `shader_chunks_cli_test.rs::list_chunks_lists_all_four_bundled_chunks_with_expected_columns` |

There is no dedicated serialization step for `ChunkName` — it is a bare
`String`/`&str` throughout (`docs/cli/type/01_chunk_name.md`'s disclosed
simplification), so "serialization" collapses into TC-3's round-trip
check rather than a distinct case.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 4 |
| Real test functions referenced | 6 (3 shared across cases) |
| Construction/parsing | TC-1, TC-4 |
| Invalid-input rejection | TC-2 |
| Round-trip | TC-3 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/01_name.md`](../param/01_name.md) | `name` parameter — concrete usage context for this type |
| [`../param/02_names.md`](../param/02_names.md) | `names` parameter — list-of-this-type usage context |
