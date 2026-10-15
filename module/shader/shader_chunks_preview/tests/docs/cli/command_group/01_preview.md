# Command Group Test :: Preview

Source: [`../../../../docs/cli/command_group/01_preview.md`](../../../../docs/cli/command_group/01_preview.md)

### Group Cases (CG-N)

| ID | Invariant | Evidence |
|----|-----------|----------|
| CG-1 | A bundled chunk resolves to a naga-validated bundle with a non-empty slider list | `shader_chunks_preview/tests/preview_cli_test.rs::name_target_prepares_a_validated_bundle`; `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_serve_0_succeeds_and_prints_the_summary` |
| CG-2 | An unknown chunk name fails loudly (shared `UnknownChunk` text), exit 1, in-process and via subprocess | `shader_chunks_preview/tests/preview_cli_test.rs::unknown_name_is_rejected_with_the_shared_unknown_chunk_text`; `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_unknown_name_fails_with_exit_1` |
| CG-3 | An unreadable `file::` target fails loudly with exit 2, never a panic | `shader_chunks_preview/tests/preview_cli_test.rs::missing_file_is_an_io_error_with_exit_code_2` |
| CG-4 | Exactly one target (`name` xor `file::`) is required — omitting both, or giving both, fails with exit 1 before any lookup | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_no_target_fails_loudly`; `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_both_targets_fails_loudly` |
| CG-5 | `serve::0` writes a bundle into the web runner crate that round-trips to the same target, and the process succeeds without a browser hand-off | `shader_chunks_preview/tests/preview_cli_test.rs::preview_without_serve_writes_the_bundle_into_the_web_runner_crate`; `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_serve_0_succeeds_and_prints_the_summary` |
| CG-6 | The help screen renders this group with exactly its documented membership (`preview`) | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_help_lists_the_preview_group` |

### Membership Coverage

Confirms the group's Semantic Coherence Test
("[`01_preview.md`](../../../../docs/cli/command_group/01_preview.md#semantic-coherence-test)")
holds for the current member:

| Command | Answers | Confirmed |
|---------|---------|-----------|
| `.preview` | What would this chunk look like live in the browser, with its tunables wired to sliders | ✅ |

### Workflow Compositions

*Conceptual only — not independently pinned by a composed test today.*
[`parameters`](../../../../../shader_chunks_params/tests/docs/cli/command_group/01_parameters.md)'s Typical Patterns describes
`tunables <name>` (see what's tunable) then `preview <name>` (see it
live) as the natural sequence, but no test composes a tunables-then-preview
real-test pair the way [`compose`](../../../../../shader_chunks_compose/tests/docs/cli/command_group/01_compose.md) WF-1 composes
`tree` then `compose` — each command's own test suite passes
independently, but the sequence itself is undemonstrated. Filed as a
disclosed gap rather than a fabricated WF row.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 6 |
| Behaviorally tested | 6 |
| Structurally verified | 0 |
| Workflow compositions | 0 (conceptual only — see above) |
| Membership coverage | 1/1 commands |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_preview.md`](../command/cmd_001_preview.md) | Member command test spec |
| [`../../../../docs/cli/command_group/01_preview.md`](../../../../docs/cli/command_group/01_preview.md) | Group documentation source |
