# Command Group :: 6. Render

### Pattern

Static image artifact production: build a
[`shader_chunks_preview_core::PreviewBundle`](../../../../shader_chunks_preview_core/readme.md)
from a bundled chunk or a local WGSL file (reusing
[`shader_chunks_preview`](../../../../shader_chunks_preview/readme.md)'s
`bundle_prepare`, naga validation included), render one frame of it on a
headless GPU via
[`shader_chunks_render_core`](../../../../shader_chunks_render_core/readme.md),
and write the frame as a PNG at `out::`.

### Purpose

Let a shader author capture a chunk as a static, committable image —
documentation previews, visual regression fixtures, quick terminal-only
inspection on a machine without a browser — without hand-writing a WGPU
harness. The file-writing counterpart of [Preview](../../../../shader_chunks_preview/docs/cli/command_group/01_preview.md)'s live
browser session. `all::1` widens the same capture to the whole registry
in one pass, for regenerating a full fixture set at once.

### Semantic Coherence Test

"The member command produces a finished static image file from the
compiled-in chunk registry (or a local file), with no server, browser, or
ongoing process involved." `.render` is the only command whose output
artifact is an image on disk — not stdout content, not a live dev
server.

### Why NOT Merge Into Preview

Both `.preview` and `.render` build the identical bundle through the
identical `bundle_prepare` path — but their artifacts and runtime shapes
diverge exactly where grouping matters: [Preview](../../../../shader_chunks_preview/docs/cli/command_group/01_preview.md)'s
contract is a *live session* (a written bundle consumed by the
`shader_chunks_preview_web` runner, a blocking dev-server subprocess, a
browser tab, sliders animating `time` continuously), while `.render`'s
contract is a *finished file* (one frozen frame, no subprocess, no
browser, process exits immediately). Merging them would make one
command's success mean either "a server is now running" or "a file now
exists" depending on flags — two different artifact species under one
name. The shared bundle-building lineage lives in code (both call
`bundle_prepare`), not in the group boundary.

### Invariants

- Rendered pixels are deterministic for identical input (same chunk or
  file content, same `size::`, same `time::`, same `set::` overrides, if
  any) on a given GPU adapter — parameters take their initial values
  unless overridden via `set::`, and nothing else varies — but the
  command is NOT side-effect-free: the `out::` path is always
  (re)written on success, unconditionally overwriting any previous file
  there.
- Under `all::1`, one chunk's failure (unpreviewable shape, naga
  validation, GPU, io) never aborts the batch — every other chunk is
  still attempted, and the command's own exit code reflects only whether
  any chunk actually failed (never for a merely-skipped, unpreviewable
  one).
- Naga validation runs before any GPU work and any write — a chunk that
  fails to parse or validate leaves the `out::` path untouched and exits
  non-zero (1); a failed render never writes a partial PNG.
- Exactly one target is required: `name` (positional) or `file::`, never
  both, never neither — violating this fails loudly with exit 1 before
  any chunk lookup or file read is attempted.
- `.render` touches no other crate's directory and spawns no subprocess —
  unlike `.preview`, which writes into `shader_chunks_preview_web` and
  hands off to `action/browser_serve`.

### Referenced Commands

| # | Command | Relationship |
|---|---------|---------------|
| 1 | [`.render`](../command/01_render.md) | Member — one-frame headless PNG render of one chunk, or every previewable chunk at once via `all::1` |

**Membership:** 1 of the 9 commands across the `shader_chunks` family; the
full partition across all 7 command groups (spanning all 6 leaf CLIs) is
stated in [the family index](../../../../shader_chunks/docs/cli/readme.md).
A single-member group is deliberate — the boundary is output-species (a
finished static image file), not command count.

### Referenced Tests

| File | Relationship |
|------|--------------|
| [`../../../tests/docs/cli/command_group/01_render.md`](../../../tests/docs/cli/command_group/01_render.md) | Group-level test specification |
| [`../../../../shader_chunks_render/tests/render_cli_test.rs`](../../../../shader_chunks_render/tests/render_cli_test.rs) | `size_parse_accepts_square_and_explicit_forms`, `size_parse_rejects_zero_missing_and_junk_sides`, `out_path_default_derives_from_the_target`, `name_target_renders_a_png_of_the_requested_size`, `unknown_name_is_rejected_with_the_shared_unknown_chunk_text`, `unpreviewable_chunk_is_rejected_before_any_gpu_work`, `missing_file_is_an_io_error_with_exit_code_2`, `subprocess_render_writes_the_png_and_prints_the_summary`, `subprocess_render_with_unknown_name_fails_with_exit_1`, `subprocess_render_with_no_target_fails_loudly`, `subprocess_render_with_both_targets_fails_loudly`, `subprocess_render_with_bad_size_fails_with_exit_1`, `subprocess_render_with_non_numeric_time_is_rejected_by_coercion`, `subprocess_help_lists_the_render_group`, `set_override_replaces_the_named_parameters_default_value`, `subprocess_render_with_unknown_set_parameter_fails_with_exit_1`, `render_all_to_png_creates_the_out_dir_and_covers_every_bundled_chunk_with_no_failures`, `render_all_to_png_writes_a_valid_png_for_every_rendered_chunk`, `render_all_to_png_skips_the_known_unpreviewable_chunk_without_writing_a_file`, `batch_summary_lists_each_outcome_and_a_totals_line`, `subprocess_render_all_writes_a_png_per_chunk_into_a_freshly_created_dir_and_reports_totals`, `subprocess_render_all_rejects_a_name_target`, `subprocess_render_all_rejects_a_file_target`, `subprocess_render_all_rejects_set_overrides` |
| [`../../../../shader_chunks_render_core/tests/render_core_test.rs`](../../../../shader_chunks_render_core/tests/render_core_test.rs) | Engine-level: exact constant-color pixels, grayscale harness properties, row-padding widths, time drift, zero-size rejection, uniform layout |

### Typical Patterns

Discover with [Query](../../../../shader_chunks_query/docs/cli/command_group/01_query.md), iterate live with
[Preview](../../../../shader_chunks_preview/docs/cli/command_group/01_preview.md)'s sliders, then `render <name>` to freeze the
result as a committable image — `size::` for the final resolution,
`time::` to pick the drift instant the live preview showed. For a chunk
outside the previewable shapes, write a small fragment-stage harness
chunk and `render file::<harness>.wgsl`.

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`../readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
