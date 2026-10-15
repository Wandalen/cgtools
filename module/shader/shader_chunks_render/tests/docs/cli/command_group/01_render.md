# Command Group Test :: Render

Source: [`../../../../docs/cli/command_group/01_render.md`](../../../../docs/cli/command_group/01_render.md)

### Group Cases (CG-N)

| ID | Invariant | Evidence |
|----|-----------|----------|
| CG-1 | A bundled chunk renders to a real, decodable PNG of the requested size, in-process and via subprocess | `shader_chunks_render/tests/render_cli_test.rs::name_target_renders_a_png_of_the_requested_size`; `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_writes_the_png_and_prints_the_summary` |
| CG-2 | An unknown chunk name fails loudly (shared `UnknownChunk` text), exit 1, in-process and via subprocess | `shader_chunks_render/tests/render_cli_test.rs::unknown_name_is_rejected_with_the_shared_unknown_chunk_text`; `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_unknown_name_fails_with_exit_1` |
| CG-3 | An unreadable `file::` target fails loudly with exit 2, never a panic | `shader_chunks_render/tests/render_cli_test.rs::missing_file_is_an_io_error_with_exit_code_2` |
| CG-4 | Exactly one target (`name` xor `file::`) is required — both arms pinned: neither given and both given each fail with exit 1 before any lookup | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_no_target_fails_loudly`; `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_both_targets_fails_loudly` |
| CG-5 | Validation precedes GPU work and the write — a rejected chunk (unknown, or outside the previewable shapes) leaves the `out::` path untouched | `shader_chunks_render/tests/render_cli_test.rs::unpreviewable_chunk_is_rejected_before_any_gpu_work`; `shader_chunks_render/tests/render_cli_test.rs::unknown_name_is_rejected_with_the_shared_unknown_chunk_text` |
| CG-6 | The help screen renders this group with exactly its documented membership (`render`) | `shader_chunks_render/tests/render_cli_test.rs::subprocess_help_lists_the_render_group`; `cli_subprocess_test.rs::top_level_help_groups_commands_by_responsibility` (asserts `Preview` < `Render` group order in the aggregator) |
| CG-7 | The aggregated `shader_chunks` binary carries the full behavior — a PNG written end to end, and a loud non-panic failure | `cli_subprocess_test.rs::render_writes_a_png_through_the_aggregated_binary`; `cli_subprocess_test.rs::render_unknown_chunk_exits_non_zero_without_a_panic_backtrace` |
| CG-8 | Under `all::1`, one chunk's skip (unpreviewable shape) or failure never aborts the batch — every chunk is attempted, the summary reports per-chunk outcomes plus totals, and the batch's own exit code reflects only true failures, never a mere skip | `shader_chunks_render/tests/render_cli_test.rs::render_all_to_png_creates_the_out_dir_and_covers_every_bundled_chunk_with_no_failures`; `shader_chunks_render/tests/render_cli_test.rs::render_all_to_png_skips_the_known_unpreviewable_chunk_without_writing_a_file`; `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_all_writes_a_png_per_chunk_into_a_freshly_created_dir_and_reports_totals` |

Engine-level determinism behind CG-1 — exact constant-color pixels on
any conformant adapter, grayscale harness structure, padding-requiring
row widths, `time` drift, zero-size rejection before context creation,
and the uniform float layout shared with the browser runner — is pinned
by `shader_chunks_render_core/tests/render_core_test.rs`
(`render_of_a_constant_fragment_chunk_is_exact`,
`render_of_a_value_chunk_matches_the_synthesized_grayscale_harness`,
`render_handles_widths_whose_row_bytes_need_padding`,
`render_time_advances_the_synthesized_drift`,
`render_rejects_zero_size_before_any_gpu_work`,
`uniform_floats_packs_time_then_params_then_aligned_resolution`).

### Membership Coverage

Confirms the group's Semantic Coherence Test
("[`01_render.md`](../../../../docs/cli/command_group/01_render.md#semantic-coherence-test)")
holds for the current member:

| Command | Answers | Confirmed |
|---------|---------|-----------|
| `.render` | What does this chunk look like as a finished static image file, with no server, browser, or ongoing process | ✅ |

### Workflow Compositions

*Conceptual only — not independently pinned by a composed test today.*
[`../../../../docs/cli/command_group/01_render.md`](../../../../docs/cli/command_group/01_render.md)'s
Typical Patterns describes `preview <name>` (iterate live) then
`render <name>` (freeze the result) as the natural sequence, but no test
composes a preview-then-render real-test pair — each command's own suite
passes independently, and both build the identical bundle through the
same `bundle_prepare` call, which is what makes the sequence coherent.
Filed as a disclosed gap rather than a fabricated WF row, matching
[`preview`](../../../../../shader_chunks_preview/tests/docs/cli/command_group/01_preview.md)'s precedent.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Group cases | 8 |
| Behaviorally tested | 8 |
| Structurally verified | 0 |
| Workflow compositions | 0 (conceptual only — see above) |
| Membership coverage | 1/1 commands |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_render.md`](../command/cmd_001_render.md) | Member command test spec |
| [`../../../../docs/cli/command_group/01_render.md`](../../../../docs/cli/command_group/01_render.md) | Group documentation source |
| [`preview`](../../../../../shader_chunks_preview/tests/docs/cli/command_group/01_preview.md) | Sibling group sharing the bundle-building lineage |
