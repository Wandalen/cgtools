# Parameter Test :: size

Source: [`../../../../docs/cli/param/02_size.md`](../../../../docs/cli/param/02_size.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Accepted forms — square (`256`), explicit (`128x64`), minimum (`1`), whitespace-padded (`" 32 x 16 "`) | `shader_chunks_render/tests/render_cli_test.rs::size_parse_accepts_square_and_explicit_forms` |
| EC-2 | Rejected forms — zero sides (`0`, `0x5`, `5x0`), empty, missing sides (`64x`, `x64`, `x`), junk (`abc`), negatives, extra segments (`1x2x3`), uppercase separator (`256X256`), fractions (`1.5`) — each with the quoted offending value | `shader_chunks_render/tests/render_cli_test.rs::size_parse_rejects_zero_missing_and_junk_sides` |
| EC-3 | End-to-end rejection — `size::0` exits 1 via subprocess with the documented error text | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_bad_size_fails_with_exit_1` |
| EC-4 | The requested size is the written PNG's actual pixel dimensions | `shader_chunks_render/tests/render_cli_test.rs::name_target_renders_a_png_of_the_requested_size`; `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_writes_the_png_and_prints_the_summary` |
| EC-5 | Widths whose row bytes are not a 256-byte multiple survive readback intact (padding strip) | `shader_chunks_render_core/tests/render_core_test.rs::render_handles_widths_whose_row_bytes_need_padding` |

### Simple Co-Dependencies

Member of no [parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md). The parsed
`(width, height)` pair feeds both the render target's dimensions and the
bundle's `resolution` uniform — one value, two consumers, so a
chunk reading `params.resolution` can never disagree with the PNG's
actual size (see EC-4's dimension assertions).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 5 |
| Real test functions | 6 |
| P0 (exit-code-affecting) | EC-2, EC-3 |
| P1 (structural output) | EC-1, EC-4, EC-5 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_render.md`](../command/cmd_001_render.md) | Sole command accepting `size` |
| [`../../../../docs/cli/param/02_size.md`](../../../../docs/cli/param/02_size.md) | Parameter documentation source |
