# Command Group Test :: Query

Source: [`../../../../docs/cli/command_group/01_query.md`](../../../../docs/cli/command_group/01_query.md)

### Group Cases (CG-N)

| ID | Invariant | Evidence |
|----|-----------|----------|
| CG-1 | `list` and `get` run one shared engine: identical explicit parameters produce byte-identical output | `shader_chunks_test.rs::query_list_and_get_defaults_share_engine_and_agree_under_equal_params`; `cli_subprocess_test.rs::list_and_get_agree_under_identical_explicit_parameters` |
| CG-2 | The two query commands differ only in defaults and `names` requiredness — `get` without names fails loudly while `list` succeeds | `cli_subprocess_test.rs::get_without_names_fails_loudly_while_list_succeeds` |
| CG-3 | Every parameter validation failure exits non-zero with a loud message, never a panic | `shader_chunks_test.rs::query_enum_params_round_trip_and_reject_bogus_values`; `cli_subprocess_test.rs::invalid_param_values_exit_non_zero_loudly`; `cli_subprocess_test.rs::get_unknown_chunk_exits_non_zero_without_a_panic_backtrace` |
| CG-4 | Idempotent, no side effects, registry-only — output is a pure function of arguments | Structural: `chunks_query` and `tags_list` in `src/lib.rs` take `&QueryParams`/no input and return fresh `Result<String, CliError>`; no `std::fs`/`std::env` usage (mechanically re-checkable via `grep -rn "std::fs\|std::env" src/`) |
| CG-5 | The help screen renders this group with exactly its documented membership (`list`, `get`, `tags`) | `cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility` |

CG-4 is a structural invariant rather than behavior a unit test observes
directly — documented honestly as such, distinguished from the genuine
behavioral evidence on the other rows.

### Membership Coverage

Confirms the group's Semantic Coherence Test
("[`01_query.md`](../../../../docs/cli/command_group/01_query.md#semantic-coherence-test)")
holds for every current member:

| Command | Answers | Confirmed |
|---------|---------|-----------|
| `.list` | Which chunks match these filters, showing these fields | ✅ |
| `.get` | Same, with the candidate set fixed to named chunks | ✅ |
| `.tags` | What tags exist and on which chunks | ✅ |

### Workflow Compositions

Documented intra-group workflows, each verified by composing the steps'
own independently-passing tests — no dedicated multi-invocation test
exists, and none is needed: every member is stateless and idempotent
(CG-4), so a 2-command workflow's correctness follows from both
commands' individual correctness.

| ID | Workflow | Composed From (Real Tests) |
|----|----------|------------------------------|
| WF-1 | `list` then `get <name>` — discover a chunk, then inspect it | `cli_subprocess_test.rs::list_prints_a_table_with_all_four_bundled_chunks` + `cli_subprocess_test.rs::get_hash21_prints_one_expanded_detail_record` |
| WF-2 | `tags` then `list tag::<selector>` then `get <name>` — discover by tag, narrow, inspect | `cli_subprocess_test.rs::tags_prints_every_distinct_tag` + `cli_subprocess_test.rs::list_filters_and_formats_via_named_params` + `cli_subprocess_test.rs::get_hash21_prints_one_expanded_detail_record` |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 5 |
| Behaviorally tested | 4 (CG-1, CG-2, CG-3, CG-5) |
| Structurally verified | 1 (CG-4) |
| Workflow compositions | 2 |
| Membership coverage | 3/3 commands |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/readme.md`](../command/readme.md) | Member command test specs |
| [`../param_group/readme.md`](../param_group/readme.md) | Shared parameter surface test specs |
| [`../../../../docs/cli/command_group/01_query.md`](../../../../docs/cli/command_group/01_query.md) | Group documentation source |
