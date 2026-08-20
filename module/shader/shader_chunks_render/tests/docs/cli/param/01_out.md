# Parameter Test :: out

Source: [`../../../../docs/cli/param/01_out.md`](../../../../docs/cli/param/01_out.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Default derivation, all three arms — `<name>.png` for a name target, file stem + `.png` for a file target, explicit `out::` winning over both | `shader_chunks_render/tests/render_cli_test.rs::out_path_default_derives_from_the_target` |
| EC-2 | Explicit `out::` written end to end — the PNG exists at the given path and decodes at the requested size | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_writes_the_png_and_prints_the_summary`; `cli_subprocess_test.rs::render_writes_a_png_through_the_aggregated_binary` |
| EC-3 | Failure leaves the path untouched — an unknown chunk produces no partial or spurious PNG | `shader_chunks_render/tests/render_cli_test.rs::unknown_name_is_rejected_with_the_shared_unknown_chunk_text` |
| EC-4 | Unwritable path (missing parent directory) — write-side `RenderCliError::Io`, exit 2, `io error` Display prefix, no file left behind | `shader_chunks_render/tests/render_cli_test.rs::unwritable_out_path_is_an_io_error_with_exit_code_2` |

### Simple Co-Dependencies

Member of no [parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md) — an
artifact-path selector, not a filter/projection/format modifier. Its
default is *derived from* whichever target parameter
([`name`](../../../../../shader_chunks_query/tests/docs/cli/param/01_name.md) or [`file`](../../../../../shader_chunks_preview/tests/docs/cli/param/01_file.md)) was given —
the only parameter in the CLI whose default depends on another
parameter's value.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 4 |
| Real test functions | 5 |
| P0 (exit-code-affecting) | EC-3, EC-4 |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_render.md`](../command/cmd_001_render.md) | Sole command accepting `out` |
| [`name`](../../../../../shader_chunks_query/tests/docs/cli/param/01_name.md) | Target whose name seeds the default |
| [`file`](../../../../../shader_chunks_preview/tests/docs/cli/param/01_file.md) | Target whose stem seeds the default |
