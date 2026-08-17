# Parameter Test :: out

Source: [`../../../../docs/cli/param/01_out.md`](../../../../docs/cli/param/01_out.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `compose_write` writes the composed text verbatim and returns a `wrote <path> (<n> bytes wgsl)` summary | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_write_writes_the_composed_text_and_returns_a_byte_count_summary` |
| EC-2 | Unwritable path (missing parent directory) — `ComposeCliError::Io`, exit 2, `io error` Display prefix, no file left behind | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::compose_write_to_an_unwritable_path_is_an_io_error_with_exit_code_2` |
| EC-3 | `out::` given end to end — the file exists at the given path with the exact composed text, and stdout carries only the summary (never the WGSL itself) | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::subprocess_compose_writes_the_file_and_prints_the_summary` |
| EC-4 | `out::` omitted — composed text still goes to stdout, no summary line appears | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::subprocess_compose_without_out_prints_composed_text_to_stdout` |
| EC-5 | `out::` combined with `transitive::1` — the file receives the full dependency closure, in dependency order | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::subprocess_compose_with_out_and_transitive_writes_the_full_closure` |
| EC-6 | `out::` to an unwritable path through the subprocess boundary — exit 2, `io error` in stderr, no file left behind | `shader_chunks_compose/tests/shader_chunks_compose_test.rs::subprocess_compose_out_to_unwritable_path_fails_with_exit_2` |

### Simple Co-Dependencies

Member of no [parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md) — an
artifact-path selector, not a filter/projection/format modifier. Composes
cleanly with `transitive::` (EC-5): the write step only ever sees
already-resolved text, so it is indifferent to how that text was
assembled.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 6 |
| Real test functions | 6 |
| P0 (exit-code-affecting) | EC-2, EC-6 |
| P1 (structural output) | EC-1, EC-3, EC-4, EC-5 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_compose.md`](../command/cmd_001_compose.md) | Sole command accepting `out` |
| [`../../../../../shader_chunks_render/tests/docs/cli/param/01_out.md`](../../../../../shader_chunks_render/tests/docs/cli/param/01_out.md) | `render`'s own `out` — same error/exit-code shape, different default semantics |
