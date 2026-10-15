# Command Group Test :: Validate

Source: [`../../../../docs/cli/command_group/01_validate.md`](../../../../docs/cli/command_group/01_validate.md)

### Group Cases (CG-N)

| ID | Invariant | Evidence |
|----|-----------|----------|
| CG-1 | A clean chunk set produces zero findings | `shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs::clean_fixture_produces_no_findings` |
| CG-2 | `manifest_drift` reports a chunk whose compiled-in descriptor field disagrees with its own WGSL manifest text | `shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs::manifest_drift_is_reported_for_a_mismatched_field` |
| CG-3 | `duplicate_name` reports two chunks sharing a `//@ name:` | `shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs::duplicate_name_is_reported_for_two_chunks_sharing_a_name` |
| CG-4 | `missing_dependency` reports an absent `//@ depends_on:` target, and never double-reports it as a derivative `dependency_cycle` | `shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs::missing_dependency_is_reported_and_not_duplicated_as_a_cycle` |
| CG-5 | `dependency_cycle` reports an unsortable registry, and never double-reports it as a derivative `wgsl_compile` failure | `shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs::dependency_cycle_is_reported_and_not_duplicated_as_wgsl_compile_failure` |
| CG-6 | `wgsl_compile` reports a chunk whose transitive closure fails naga parse | `shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs::wgsl_compile_is_reported_for_a_naga_parse_failure` |
| CG-7 | `wgsl_compile` accepts a dependency-only chunk with no `@vertex`/`@fragment`/`@compute` entry point as clean | `shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs::wgsl_compile_accepts_a_dependency_only_chunk_with_no_entry_point` |
| CG-8 | The real bundled `shader_chunks_core::CHUNKS` registry reports zero findings today | `shader_chunks_validate_core/tests/shader_chunks_validate_core_test.rs::validate_registry_reports_nothing_for_the_current_bundled_registry` |
| CG-9 | Zero findings renders the CLI's explicit all-clear message end to end, never blank output | `shader_chunks_validate/tests/validate_cli_test.rs::clean_fixture_produces_the_all_clear_message`; `shader_chunks_validate/tests/validate_cli_test.rs::the_real_bundled_registry_is_reported_clean_through_the_cli_wiring` |
| CG-10 | The help screen renders this group with exactly its documented membership (`validate`) | `cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility` |

### Membership Coverage

Confirms the group's Semantic Coherence Test
("[`01_validate.md`](../../../../docs/cli/command_group/01_validate.md#semantic-coherence-test)")
holds for every current member:

| Command | Answers | Confirmed |
|---------|---------|-----------|
| `.validate` | Whether the whole compiled-in chunk set is internally consistent and actually compiles | ✅ |

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 10 |
| Behaviorally tested | 10 |
| Structurally verified | 0 |
| Membership coverage | 1/1 commands |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_validate.md`](../command/cmd_001_validate.md) | Member command test spec |
| [`../../../../docs/cli/command_group/01_validate.md`](../../../../docs/cli/command_group/01_validate.md) | Group documentation source |
