# Command Test :: tags

Source: [`../../../../docs/cli/command/03_tags.md`](../../../../docs/cli/command/03_tags.md)

### Parameter Edge Tests (PAR-N)

*N/A — `.tags` declares zero parameters (see
[`../../../../docs/cli/command/03_tags.md`](../../../../docs/cli/command/03_tags.md)'s
Parameters table).*

### Parameter Group Corner Tests (GRP-N)

*N/A — `.tags` declares zero parameters, so none of the CLI's
[parameter groups](../param_group/readme.md) applies to it.*

### Integration Tests (INT-N)

| ID | Scenario | Real Test |
|----|----------|-----------|
| INT-1 | Direct call lists every distinct `group:tag` pair and its carrying chunk(s) | `shader_chunks_test.rs::list_tags_lists_every_distinct_group_tag_pair_and_its_chunks` |
| INT-2 | Subprocess invocation prints every distinct tag | `cli_subprocess_test.rs::tags_prints_every_distinct_tag` |

See also [`../command_group/01_query.md`](../command_group/01_query.md)
WF-2 for `tags`'s role preceding `list tag::`/`get` in a
discover-by-tag-then-inspect workflow.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 0 (no parameters) |
| GRP-N | 0 (no parameters, so no group applies) |
| INT-N | 2 |

### See Also

- [`../../../../docs/cli/command/03_tags.md`](../../../../docs/cli/command/03_tags.md) — command source
- [`../../../../docs/cli/format/01_table_plain.md`](../../../../docs/cli/format/01_table_plain.md) — output format
