# Parameter Test :: all

Source: [`../../../../docs/cli/param/05_all.md`](../../../../docs/cli/param/05_all.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `all::` omitted — behaves exactly as before this parameter existed, rendering exactly one resolved target | `shader_chunks_render/tests/render_cli_test.rs::name_target_renders_a_png_of_the_requested_size` |
| EC-2 | `render_all_to_png` creates a missing `out_dir` and covers every entry in `shader_chunks_core::CHUNKS` with no failures | `shader_chunks_render/tests/render_cli_test.rs::render_all_to_png_creates_the_out_dir_and_covers_every_bundled_chunk_with_no_failures` |
| EC-3 | `render_all_to_png` writes a structurally valid PNG for every chunk it renders | `shader_chunks_render/tests/render_cli_test.rs::render_all_to_png_writes_a_valid_png_for_every_rendered_chunk` |
| EC-4 | `render_all_to_png` skips the known unpreviewable chunk without writing a file for it, and without failing the batch | `shader_chunks_render/tests/render_cli_test.rs::render_all_to_png_skips_the_known_unpreviewable_chunk_without_writing_a_file` |
| EC-5 | `batch_summary` lists each per-chunk outcome plus a totals line (`<n> chunks: <r> rendered, <s> skipped, <f> failed`) | `shader_chunks_render/tests/render_cli_test.rs::batch_summary_lists_each_outcome_and_a_totals_line` |
| EC-6 | Subprocess boundary: `all::1` through the real CLI writes one PNG per chunk into a freshly created directory and reports totals in stdout | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_all_writes_a_png_per_chunk_into_a_freshly_created_dir_and_reports_totals` |
| EC-7 | Subprocess boundary: `all::1` combined with a `name` target is rejected, exit 1 | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_all_rejects_a_name_target` |
| EC-8 | Subprocess boundary: `all::1` combined with a `file::` target is rejected, exit 1 | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_all_rejects_a_file_target` |
| EC-9 | Subprocess boundary: `all::1` combined with `set::` overrides is rejected, exit 1 | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_all_rejects_set_overrides` |

EC-2 through EC-5 exercise `render_all_to_png`/`batch_summary` directly,
in-process; EC-6 through EC-9 pin the identical semantics through the
unilang `Kind::Boolean` coercion and CLI dispatch boundary, including the
three-way mutual-exclusivity rejection.

### Simple Co-Dependencies

Member of no [parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md).
The only parameter that switches the command from single-target to
full-registry-sweep mode — EC-7 through EC-9 test that combining it with
any of the three target-adjacent parameters (`name`, `file::`, `set::`) is
rejected outright before any chunk is touched, the same
reject-before-any-work discipline `set`'s own EC-4/EC-5 apply at the
element-shape stage (see [`04_set.md`](04_set.md)).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 9 |
| Real test functions | 9 |
| P0 (exit-code-affecting) | EC-7, EC-8, EC-9 |
| P1 (structural output) | EC-1, EC-2, EC-3, EC-4, EC-5, EC-6 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_render.md`](../command/cmd_001_render.md) | Sole command accepting `all` |
| [`../../../../../shader_chunks_query/tests/docs/cli/type/07_switch.md`](../../../../../shader_chunks_query/tests/docs/cli/type/07_switch.md) | Underlying Switch-type contract |
