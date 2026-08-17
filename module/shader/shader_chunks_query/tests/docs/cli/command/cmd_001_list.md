# Command Test :: list

Source: [`../../../../docs/cli/command/01_list.md`](../../../../docs/cli/command/01_list.md)

### Parameter Edge Tests (PAR-N)

Command-level parameter behavior — per-parameter boundary detail lives
in the 21 [parameter mirrors](../param/readme.md) (`02_names`,
`03_pattern` through `21_width`, and `23_source`), all applicable to
`list`.

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | All defaults: bare `list` renders every chunk as a plain table | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_list_defaults_renders_every_chunk_as_plain_table`; `cli_subprocess_test.rs::list_prints_a_table_with_all_four_bundled_chunks` |
| PAR-2 | `names` optional: omitting it selects the full registry, never errors | `cli_subprocess_test.rs::get_without_names_fails_loudly_while_list_succeeds` (list arm) |
| PAR-3 | Named filter/format params bind end-to-end (`tag::`, `roots::`, `count::`, `format::names`) | `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` |
| PAR-4 | Invalid values for closed-set/integer params exit non-zero loudly | `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly` |

### Parameter Group Corner Tests (GRP-N)

Group-interaction corner cases live in the
[parameter group mirrors](../param_group/readme.md) —
[filtering](../param_group/01_filtering.md) GRP-1..6,
[projection](../param_group/02_projection.md) GRP-1..4,
[formatting](../param_group/03_formatting.md) GRP-1..5. Their cited
tests run through the shared query engine `list` binds, so each applies
to this command verbatim.

### Integration Tests (INT-N)

| ID | Scenario | Real Test |
|----|----------|-----------|
| INT-1 | Identical explicit params on `list` and `get` — byte-identical output (one engine) | `cli_subprocess_test.rs::list_and_get_agree_under_identical_explicit_parameters` |
| INT-2 | Per-command help lists every named param with `list`'s own defaults | `cli_subprocess_test.rs::per_command_help_lists_named_params_with_per_command_defaults` |
| INT-3 | Top-level help renders `list` under the Query group | `cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility` |

Discover-then-inspect workflows: [`../command_group/01_query.md`](../command_group/01_query.md) WF-1/WF-2.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 4 (+21 delegated parameter mirrors) |
| GRP-N | delegated (16 GRP cases across 3 groups) |
| INT-N | 3 |

### See Also

- [`../../../../docs/cli/command/01_list.md`](../../../../docs/cli/command/01_list.md) — command source
- [`../command_group/01_query.md`](../command_group/01_query.md) — group invariants (shared engine, help grouping)
- [`../../../../docs/cli/format/01_table_plain.md`](../../../../docs/cli/format/01_table_plain.md) — default output format
