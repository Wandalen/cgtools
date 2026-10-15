# Type Test :: Float

Source: [`../../../../docs/cli/type/01_float.md`](../../../../docs/cli/type/01_float.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Default value: `0` passes `Kind::Float` coercion and the finiteness guard, reaching a successful render | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_writes_the_png_and_prints_the_summary` |
| TC-2 | Non-numeric rejection: unilang's `Kind::Float` coercion fails before the routine is entered, non-zero exit | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_non_numeric_time_is_rejected_by_coercion` |
| TC-3 | Value semantics: the `f32` actually drives behavior — two values of the sole `Float`-typed parameter produce visibly different output | `shader_chunks_render_core/tests/render_core_test.rs::render_time_advances_the_synthesized_drift` |
| TC-4 | Token-shape acceptance: explicit fractional (`2.5`) and integer-shaped (`2`) tokens both coerce — the `Value::Float`/`Value::Integer` arms of `arg_time` | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_fractional_time_succeeds`; `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_integer_time_token_succeeds` |
| TC-5 | Non-finite rejection: `inf` exits non-zero and produces no artifact — the finiteness boundary of the contract | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_non_finite_time_is_rejected` |

TC-2's subprocess case drives the coercion layer shared by every
`Kind::Float` argument; TC-4/TC-5 pin the crate-local `arg_time` arms
beyond it. Per-parameter edge-case detail remains owned by
[`../param/03_time.md`](../param/03_time.md).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 5 |
| Real test functions referenced | 6 |
| Positive semantics | TC-1, TC-3, TC-4 |
| Invalid-input rejection | TC-2, TC-5 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/03_time.md`](../param/03_time.md) | Usage: frame-capture instant |
| [`NonNegativeInteger`](../../../../../shader_chunks_query/tests/docs/cli/type/08_non_negative_integer.md) | Contrasting count-shaped numeric type |
