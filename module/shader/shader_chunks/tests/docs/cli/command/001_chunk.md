# Category Integration Test :: chunk

Source: [`../../../../docs/cli/command_group.md`](../../../../docs/cli/command_group.md) (the `chunk` category = the `Inspection` command group)

### Integration Tests

| ID | Scenario | Composed From (Real Tests) |
|----|----------|------------------------------|
| INT-1 | No arguments at all → top-level help printed, exit 0 | `cli_subprocess_test.rs::no_arguments_prints_help_and_exits_zero` |
| INT-2 | `list` then `get <name>` — discover a chunk, then inspect it | `cli_subprocess_test.rs::list_prints_a_table_with_all_four_bundled_chunks` + `cli_subprocess_test.rs::get_hash21_prints_full_detail` |
| INT-3 | `tags` then `get <name>` — discover a chunk by tag, then inspect it | `cli_subprocess_test.rs::tags_prints_every_distinct_tag` + `cli_subprocess_test.rs::get_hash21_prints_full_detail` |
| INT-4 | `tree <name>` then `compose <name...>` — preview dependency order, then compose using that order | `cli_subprocess_test.rs::tree_fbm3_shows_the_dependency_chain` + `cli_subprocess_test.rs::compose_hash21_value_noise_prints_composed_wgsl_in_dependency_order` |

INT-2 through INT-4 are documented usage patterns verified by composing
each step's own independently-passing single-command test — no dedicated
multi-invocation test function exists, and none is needed: each command
is stateless and idempotent (`command_group.md` § Invariants), so a
2-command workflow's correctness follows directly from both commands'
individual correctness.

### Command-Specific Behavior

Every command in this category shares identical error-reporting
vocabulary (`CliError::UnknownChunk`/`CliError::Compose` map to the exit
codes documented per-command) — no command overrides or special-cases
this behavior.

### Real-World Scenarios

A shader author browses the registry (`list`/`tags`), inspects a
candidate chunk (`get`), checks its dependency chain (`tree`), and
previews the final composed WGSL (`compose`) before wiring
`shader_chunks_core::try_compose` into a real render pipeline.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Integration scenarios | 4 |
| Real test functions referenced | 5 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../param/01_name.md`](../param/01_name.md) | `name` parameter used in INT-2/INT-3/INT-4 |
| [`../param/02_names.md`](../param/02_names.md) | `names` parameter used in INT-4 |
