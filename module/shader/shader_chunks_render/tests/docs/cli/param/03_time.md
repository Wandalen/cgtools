# Parameter Test :: time

Source: [`../../../../docs/cli/param/03_time.md`](../../../../docs/cli/param/03_time.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Default `0` flows through unilang `Kind::Float` coercion into a successful render | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_writes_the_png_and_prints_the_summary` |
| EC-2 | Non-numeric token — rejected by unilang's float coercion before the routine runs, non-zero exit | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_non_numeric_time_is_rejected_by_coercion` |
| EC-3 | The value is semantically live — two different `time` values produce visibly different frames of the same chunk | `shader_chunks_render_core/tests/render_core_test.rs::render_time_advances_the_synthesized_drift` |
| EC-4 | Explicit fractional token (`time::2.5`) through the subprocess boundary — succeeds, summary echoes `time: 2.5`, PNG written | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_fractional_time_succeeds` |
| EC-5 | Integer-shaped token (`time::2`) — the `Value::Integer` acceptance arm of `arg_time` coerces and succeeds, summary echoes `time: 2` | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_integer_time_token_succeeds` |
| EC-6 | Non-finite token (`time::inf`) — rejected non-zero (coercion layer or the routine's finiteness guard, whichever catches it) and no PNG is written | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_non_finite_time_is_rejected` |

EC-3 exercises fractional/advanced time at the engine level
(`render(bundle, size, 10.0)`); EC-4/EC-5 pin the same semantics through
unilang coercion at the subprocess boundary.

### Simple Co-Dependencies

Member of no [parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md). The only
`Kind::Float` parameter in the CLI — the parsed value is narrowed to
`f32` and written into uniform slot 0, the same slot the browser
preview animates continuously; every other uniform slot comes from the
chunk's declared tunables at their defaults, which `time::` never
touches.

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
| [`../command/cmd_001_render.md`](../command/cmd_001_render.md) | Sole command accepting `time` |
| [`../type/01_float.md`](../type/01_float.md) | Underlying float contract |
