# Command Test :: get

Source: [`../../../../docs/cli/command/02_get.md`](../../../../docs/cli/command/02_get.md)

### Parameter Edge Tests (PAR-N)

Command-level parameter behavior — per-parameter boundary detail lives
in the 20 [parameter mirrors](../param/readme.md) (`02_names` and
`03_pattern` through `21_width`), all applicable to `get`.

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | Defaults: `get <name>` renders an expanded record with the detail field set | `shader_chunks_test.rs::query_get_defaults_renders_expanded_records_with_detail_fields`; `cli_subprocess_test.rs::get_hash21_prints_one_expanded_detail_record` |
| PAR-2 | `names` required: omitting it fails loudly (`required argument 'names' is missing`), exit 1 | `cli_subprocess_test.rs::get_without_names_fails_loudly_while_list_succeeds` |
| PAR-3 | Unknown chunk name — `CliError::UnknownChunk`, non-zero exit, no panic backtrace | `shader_chunks_test.rs::query_unknown_name_reports_unknown_chunk_error`; `cli_subprocess_test.rs::get_unknown_chunk_exits_non_zero_without_a_panic_backtrace` |
| PAR-4 | Selection keeps the given order and allows duplicates | `shader_chunks_test.rs::query_names_selects_in_given_order_and_allows_duplicates` |

Full `names` edge-case detail: [`../param/02_names.md`](../param/02_names.md) EC-1/EC-2/EC-3.

### Parameter Group Corner Tests (GRP-N)

Group-interaction corner cases live in the
[parameter group mirrors](../param_group/readme.md) —
[filtering](../param_group/01_filtering.md),
[projection](../param_group/02_projection.md), and
[formatting](../param_group/03_formatting.md). Their cited tests run
through the shared query engine `get` binds, so each applies to this
command verbatim; only the defaults differ.

### Integration Tests (INT-N)

| ID | Scenario | Real Test |
|----|----------|-----------|
| INT-1 | Identical explicit params on `list` and `get` — byte-identical output (one engine) | `cli_subprocess_test.rs::list_and_get_agree_under_identical_explicit_parameters`; `shader_chunks_test.rs::query_list_and_get_defaults_share_engine_and_agree_under_equal_params` |
| INT-2 | Per-command help lists every named param with `get`'s own defaults | `cli_subprocess_test.rs::per_command_help_lists_named_params_with_per_command_defaults` |
| INT-3 | Top-level help renders `get` under the Query group | `cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility` |

Discover-then-inspect workflows: [`../command_group/01_query.md`](../command_group/01_query.md) WF-1/WF-2.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 4 (+20 delegated parameter mirrors) |
| GRP-N | delegated (15 GRP cases across 3 groups) |
| INT-N | 3 |

### See Also

- [`../../../../docs/cli/command/02_get.md`](../../../../docs/cli/command/02_get.md) — command source
- [`../command_group/01_query.md`](../command_group/01_query.md) — group invariants (shared engine, requiredness split)
- [`../param/02_names.md`](../param/02_names.md) — `names` parameter
- [`../../../../docs/cli/format/05_expanded.md`](../../../../docs/cli/format/05_expanded.md) — default output format
