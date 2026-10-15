# Parameter Test :: serve

Source: [`../../../../docs/cli/param/02_serve.md`](../../../../docs/cli/param/02_serve.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `serve::0` writes the bundle and returns without blocking on the browser dev server | `shader_chunks_preview/tests/preview_cli_test.rs::preview_without_serve_writes_the_bundle_into_the_web_runner_crate`; `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_serve_0_succeeds_and_prints_the_summary` |
| EC-2 | Invalid coercion (`serve::maybe`) rejected non-zero by unilang before the routine runs — guarded by a 30 s timeout so a silently-accepted value that blocks on the browser server fails loudly (a timeout kill yields exit code `None`) | `shader_chunks_preview/tests/preview_cli_test.rs::subprocess_preview_with_bad_serve_value_is_rejected_by_coercion` |

### Simple Co-Dependencies

Member of no [parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md) — a
side-effect toggle (browser hand-off), not a filter/projection/format
modifier. The only `Switch` parameter in the CLI defaulting `true`
rather than `false`; `serve::0` never skips naga validation, which
happens in `bundle_prepare` before `serve()` is ever reached.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 2 |
| Real test functions | 3 |
| P0 (exit-code-affecting) | EC-2 |
| P1 (structural output) | EC-1 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_preview.md`](../command/cmd_001_preview.md) | Sole command accepting `serve` |
| [`Switch`](../../../../../shader_chunks_query/tests/docs/cli/type/07_switch.md) | Underlying boolean contract |
| [`01_file.md`](01_file.md) | Sibling target selector (shared with `.render`) |
