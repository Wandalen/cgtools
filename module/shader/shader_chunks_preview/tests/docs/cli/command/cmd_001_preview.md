# Command Test :: preview

Source: [`../../../../docs/cli/command/01_preview.md`](../../../../docs/cli/command/01_preview.md)

### Parameter Edge Tests (PAR-N)

In-process, function-level cases (no subprocess spawn):

| ID | Case | Real Test |
|----|------|-----------|
| PAR-1 | `name` target resolves and builds a naga-validated bundle with a non-empty slider list | `shader_chunks_preview/tests/preview_cli_test.rs::name_target_prepares_a_validated_bundle` |
| PAR-2 | Unknown chunk name — `PreviewCliError::UnknownChunk`, exit 1, shared unknown-chunk text | `shader_chunks_preview/tests/preview_cli_test.rs::unknown_name_is_rejected_with_the_shared_unknown_chunk_text` |
| PAR-3 | `file::` target unreadable — `PreviewCliError::Io`, exit 2 | `shader_chunks_preview/tests/preview_cli_test.rs::missing_file_is_an_io_error_with_exit_code_2` |
| PAR-4 | `serve::0` writes `-preview.json` into the web runner crate and the written JSON round-trips to the same target | `shader_chunks_preview/tests/preview_cli_test.rs::preview_without_serve_writes_the_bundle_into_the_web_runner_crate` |
| PAR-5 | `file::` target readable — a bundled chunk's own text via a temp file builds the identical bundle name mode builds | `shader_chunks_preview/tests/preview_cli_test.rs::file_target_prepares_the_same_bundle_as_the_bundled_name` |

Full `file`/`serve` edge-case detail: [`../param/01_file.md`](../param/01_file.md),
[`../param/02_serve.md`](../param/02_serve.md).

### Parameter Group Corner Tests (GRP-N)

*N/A — `preview`'s `name`/`file`/`serve` parameters belong to no
[parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md) (target selection and a
side-effect toggle, not filtering/projection/formatting), so no
within-group combination exists to corner-test.*

### Integration Tests (INT-N)

Subprocess-level, end-to-end cases:

| ID | Scenario | Real Test |
|----|----------|-----------|
| INT-1 | `preview fbm3 serve::0` succeeds; stdout contains `naga-validated` and the synthesized `preview_scale` slider name | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_serve_0_succeeds_and_prints_the_summary` |
| INT-2 | `preview bogus_chunk serve::0` fails with exit 1 and the shared unknown-chunk stderr text | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_unknown_name_fails_with_exit_1` |
| INT-3 | `preview serve::0` (no `name`, no `file::`) fails with exit 1 and an "exactly one target" stderr message | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_no_target_fails_loudly` |
| INT-4 | `help` lists the `Preview` group with a `preview [name]` entry | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_help_lists_the_preview_group` |
| INT-5 | `preview fbm3 file::whatever.wgsl serve::0` (both targets) fails with exit 1 and the "exactly one target" stderr message — the other arm of INT-3's mutual-exclusivity check | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_both_targets_fails_loudly` |
| INT-6 | `preview fbm3 serve::maybe` is rejected non-zero by unilang's boolean coercion — guarded by a 30 s timeout so a silently-accepted value that blocks on the browser server fails the test loudly (a timeout kill yields exit code `None`) | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_bad_serve_value_is_rejected_by_coercion` |

See also [`../command_group/01_preview.md`](../command_group/01_preview.md)
for this command's (single-member) group invariants.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| PAR-N | 5 |
| GRP-N | 0 (no within-group combination available) |
| INT-N | 6 |

### See Also

- [`../../../../docs/cli/command/01_preview.md`](../../../../docs/cli/command/01_preview.md) — command source
- [`../param/01_file.md`](../param/01_file.md) — `file` parameter
- [`../param/02_serve.md`](../param/02_serve.md) — `serve` parameter
- [`plain_text`](../../../../../shader_chunks_compose/docs/cli/format/01_plain_text.md) — output format
