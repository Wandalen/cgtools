# Parameter Test :: file

Source: [`../../../../docs/cli/param/01_file.md`](../../../../docs/cli/param/01_file.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Unreadable/missing path (`.preview`) — `PreviewCliError::Io`, exit 2 | `shader_chunks_preview/tests/preview_cli_test.rs::missing_file_is_an_io_error_with_exit_code_2` |
| EC-2 | Absent alongside an absent `name` (`.preview`) — exactly one target required, exit 1, "exactly one target" text | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_no_target_fails_loudly` |
| EC-3 | Unreadable/missing path (`.render`) — the same `Io` mapping through `RenderCliError::Preview`, exit 2 | `shader_chunks_render/tests/render_cli_test.rs::missing_file_is_an_io_error_with_exit_code_2` |
| EC-4 | Both `name` and `file::` given (`.render`) — exit 1, "exactly one target" text | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_both_targets_fails_loudly` |
| EC-5 | Neither target given (`.render`) — exit 1, same text | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_no_target_fails_loudly` |
| EC-6 | Both `name` and `file::` given (`.preview`) — exit 1, "exactly one target" text | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_both_targets_fails_loudly` |
| EC-7 | Successful read (`.preview`) — a bundled chunk's own text written to a temp file prepares the *identical* bundle via `file::` as via `name` | `shader_chunks_preview/tests/preview_cli_test.rs::file_target_prepares_the_same_bundle_as_the_bundled_name` |
| EC-8 | Successful read (`.render`) — the same temp-file round-trip renders a naga-validated PNG of the requested size | `shader_chunks_render/tests/render_cli_test.rs::file_target_renders_the_same_chunk_text_as_a_bundled_name` |

The successful-read tests (EC-7, EC-8) avoid hand-built fixtures: they
feed `shader_chunks_core::chunk_get( "fbm3" ).wgsl` back through a temp
file, proving file mode traverses the exact pipeline name mode does —
manifest parse, dependency composition, naga validation, slider
synthesis — without maintaining a parallel `.wgsl` fixture that could
drift from the manifest grammar.

### Simple Co-Dependencies

Member of no [parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md) — like
`name`/`names`, it is a target selector, mutually exclusive with `name`
rather than co-filtering alongside it. Consumed by `.preview` and
`.render`, with identical resolution and failure modes (both call
`shader_chunks_preview::bundle_prepare`); on `.render` the file's stem
additionally seeds [`out`](../../../../../shader_chunks_render/tests/docs/cli/param/01_out.md)'s default.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 8 |
| Real test functions | 8 |
| P0 (exit-code-affecting) | EC-1, EC-2, EC-3, EC-4, EC-5, EC-6 |
| P1 (structural output) | EC-7, EC-8 (successful-read round-trips) |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_preview.md`](../command/cmd_001_preview.md) | Preview command accepting `file` |
| [`render`](../../../../../shader_chunks_render/tests/docs/cli/command/cmd_001_render.md) | Render command accepting `file` |
| [`name`](../../../../../shader_chunks_query/tests/docs/cli/param/01_name.md) | Mutually exclusive alternative target |
| [`02_serve.md`](02_serve.md) | Sibling `.preview` parameter |
| [`out`](../../../../../shader_chunks_render/tests/docs/cli/param/01_out.md) | Render output path seeded by this parameter's stem |
