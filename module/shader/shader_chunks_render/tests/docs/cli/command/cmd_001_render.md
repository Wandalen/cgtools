# Command Test :: render

Source: [`../../../../docs/cli/command/01_render.md`](../../../../docs/cli/command/01_render.md)

### Parameter Edge Tests (PAR-N)

In-process, function-level cases (no subprocess spawn):

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | `name` target renders a real, decodable PNG at the requested size; summary carries the target, `naga-validated`, and the default `preview_scale` value | `shader_chunks_render/tests/render_cli_test.rs::name_target_renders_a_png_of_the_requested_size` |
| PAR-2 | Unknown chunk name — shared unknown-chunk text, exit 1, and the out path stays absent (no partial PNG) | `shader_chunks_render/tests/render_cli_test.rs::unknown_name_is_rejected_with_the_shared_unknown_chunk_text` |
| PAR-3 | `file::` target unreadable — `Io`, exit 2 | `shader_chunks_render/tests/render_cli_test.rs::missing_file_is_an_io_error_with_exit_code_2` |
| PAR-4 | Chunk outside the previewable shapes (`hash22`, returns `vec2f`) rejected before any GPU work, exit 1 | `shader_chunks_render/tests/render_cli_test.rs::unpreviewable_chunk_is_rejected_before_any_gpu_work` |
| PAR-5 | `size::` grammar — square, explicit `<w>x<h>`, and whitespace-padded forms accepted; zero sides, missing sides, junk, uppercase `X`, fractions, and extra segments rejected with the quoted value | `shader_chunks_render/tests/render_cli_test.rs::size_parse_accepts_square_and_explicit_forms`; `shader_chunks_render/tests/render_cli_test.rs::size_parse_rejects_zero_missing_and_junk_sides` |
| PAR-6 | `out::` default derivation — `<name>.png` for a name target, file stem + `.png` for a file target, explicit `out::` winning over both | `shader_chunks_render/tests/render_cli_test.rs::out_path_default_derives_from_the_target` |
| PAR-7 | `file::` target readable — a bundled chunk's own text via a temp file renders a decodable PNG of the requested size, `naga-validated` in the summary | `shader_chunks_render/tests/render_cli_test.rs::file_target_renders_the_same_chunk_text_as_a_bundled_name` |
| PAR-8 | `out::` unwritable (missing parent directory) — write-side `RenderCliError::Io`, exit 2, no file left behind | `shader_chunks_render/tests/render_cli_test.rs::unwritable_out_path_is_an_io_error_with_exit_code_2` |
| PAR-9 | Valid `set::` overrides replace their parameters' defaults in the summary | `shader_chunks_render/tests/render_cli_test.rs::set_override_replaces_the_named_parameters_default_value` |
| PAR-10 | Unknown `set::` property rejected, exit 1, message listing every valid property | `shader_chunks_render/tests/render_cli_test.rs::set_override_rejects_an_unknown_parameter_name` |
| PAR-11 | `set::` token missing its `:` separator rejected before any bundle is built | `shader_chunks_render/tests/render_cli_test.rs::overrides_parse_rejects_a_token_missing_its_separator` |
| PAR-12 | `set::` value side non-finite or non-numeric rejected | `shader_chunks_render/tests/render_cli_test.rs::overrides_parse_rejects_a_non_finite_or_non_numeric_value` |
| PAR-13 | Two `set::` overrides of the same property — the later one wins | `shader_chunks_render/tests/render_cli_test.rs::overrides_apply_lets_a_later_override_of_the_same_property_win` |

Full parameter edge-case detail: [`file`](../../../../../shader_chunks_preview/tests/docs/cli/param/01_file.md),
[`../param/01_out.md`](../param/01_out.md),
[`../param/02_size.md`](../param/02_size.md),
[`../param/03_time.md`](../param/03_time.md),
[`../param/04_set.md`](../param/04_set.md).

Engine-level pixel guarantees (exact constant-color bytes, grayscale
harness properties, row-padding widths, time drift, zero-size rejection,
uniform layout) are pinned separately in
`shader_chunks_render_core/tests/render_core_test.rs` — cited from
[`../command_group/01_render.md`](../command_group/01_render.md).

### Parameter Group Corner Tests (GRP-N)

*N/A — `render`'s parameters (the `name`/`file` target selectors plus
`out`/`size`/`time`) belong to no
[parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md) (target selection and
artifact shaping, not filtering/projection/formatting), so no
within-group combination exists to corner-test.*

### Integration Tests (INT-N)

Subprocess-level, end-to-end cases:

| ID | Scenario | Real Test |
|----|----------|-----------|
| INT-1 | `render <name> out::<tmp> size::16` writes a decodable 16×16 PNG and prints the `16x16 px` summary | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_writes_the_png_and_prints_the_summary` |
| INT-2 | Unknown name — exit 1 with the shared unknown-chunk stderr text | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_unknown_name_fails_with_exit_1` |
| INT-3 | No `name`, no `file::` — exit 1 with an "exactly one target" stderr message | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_no_target_fails_loudly` |
| INT-4 | Both `name` and `file::` — exit 1 with the same "exactly one target" message | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_both_targets_fails_loudly` |
| INT-5 | `size::0` — exit 1 quoting the offending value | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_bad_size_fails_with_exit_1` |
| INT-6 | Non-numeric `time::` — rejected by unilang `Kind::Float` coercion, non-zero exit | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_non_numeric_time_is_rejected_by_coercion` |
| INT-7 | `help` lists the `Render` group with a `render [name]` entry | `shader_chunks_render/tests/render_cli_test.rs::subprocess_help_lists_the_render_group` |
| INT-8 | The aggregated `shader_chunks` binary renders a PNG end to end | `cli_subprocess_test.rs::render_writes_a_png_through_the_aggregated_binary` |
| INT-9 | The aggregated binary rejects an unknown chunk without a panic backtrace | `cli_subprocess_test.rs::render_unknown_chunk_exits_non_zero_without_a_panic_backtrace` |
| INT-10 | Fractional `time::2.5` succeeds; the summary echoes `time: 2.5` and the PNG is written | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_fractional_time_succeeds` |
| INT-11 | Integer-shaped `time::2` token also coerces to `Kind::Float`; the summary echoes `time: 2` | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_integer_time_token_succeeds` |
| INT-12 | Non-finite `time::inf` — rejected non-zero (by coercion or the routine's own finiteness guard, whichever layer catches it) and no PNG is written | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_non_finite_time_is_rejected` |
| INT-13 | `set::lacunarity:2.5,gain:0.75` through the subprocess boundary — succeeds, stdout shows both overridden values | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_set_override_shows_the_overridden_value` |
| INT-14 | Unknown `set::` property through the subprocess boundary — exit 1, stderr names the offending property | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_unknown_set_parameter_fails_with_exit_1` |
| INT-15 | Malformed `set::` token (no `:`) through the subprocess boundary — exit 1, stderr quotes the offending token | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_malformed_set_token_fails_with_exit_1` |

Both arms of the mutual-exclusivity check are independently pinned —
INT-3 (neither target) and INT-4 (both targets) — matching `.preview`'s
INT-3/INT-5 pair in
[`cmd_001_preview.md`](../../../../../shader_chunks_preview/tests/docs/cli/command/cmd_001_preview.md).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 13 |
| GRP-N | 0 (no within-group combination available) |
| INT-N | 15 |

### See Also

- [`../../../../docs/cli/command/01_render.md`](../../../../docs/cli/command/01_render.md) — command source
- [`../param/01_out.md`](../param/01_out.md) — `out` parameter
- [`../param/02_size.md`](../param/02_size.md) — `size` parameter
- [`../param/03_time.md`](../param/03_time.md) — `time` parameter
- [`../param/04_set.md`](../param/04_set.md) — `set` parameter
- [`../command_group/01_render.md`](../command_group/01_render.md) — group invariants + engine-level citations
- [`plain_text`](../../../../../shader_chunks_compose/docs/cli/format/01_plain_text.md) — output format
