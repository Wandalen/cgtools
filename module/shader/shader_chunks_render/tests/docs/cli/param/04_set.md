# Parameter Test :: set

Source: [`../../../../docs/cli/param/04_set.md`](../../../../docs/cli/param/04_set.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `set::` omitted — every parameter renders at its bundle-declared default, unaffected by the override machinery | `shader_chunks_render/tests/render_cli_test.rs::name_target_renders_a_png_of_the_requested_size` |
| EC-2 | Two valid overrides in one `set::` value both replace their parameters' defaults in the summary | `shader_chunks_render/tests/render_cli_test.rs::set_override_replaces_the_named_parameters_default_value` |
| EC-3 | An unrecognized property name is rejected, exit 1, message listing every property the bundle actually declares | `shader_chunks_render/tests/render_cli_test.rs::set_override_rejects_an_unknown_parameter_name` |
| EC-4 | A token missing its `:` separator is rejected before any bundle is built | `shader_chunks_render/tests/render_cli_test.rs::overrides_parse_rejects_a_token_missing_its_separator` |
| EC-5 | A non-finite (`inf`/`-inf`/`nan`) or non-numeric value side is rejected | `shader_chunks_render/tests/render_cli_test.rs::overrides_parse_rejects_a_non_finite_or_non_numeric_value` |
| EC-6 | Two overrides of the same property in one list — the later one wins | `shader_chunks_render/tests/render_cli_test.rs::overrides_apply_lets_a_later_override_of_the_same_property_win` |
| EC-7 | Subprocess boundary: `set::lacunarity:2.5,gain:0.75` through the real CLI produces the overridden values in stdout | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_set_override_shows_the_overridden_value` |
| EC-8 | Subprocess boundary: an unknown property name exits 1 with the naming error in stderr | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_unknown_set_parameter_fails_with_exit_1` |
| EC-9 | Subprocess boundary: a malformed (no `:`) token exits 1 quoting the offending token in stderr | `shader_chunks_render/tests/render_cli_test.rs::subprocess_render_with_malformed_set_token_fails_with_exit_1` |

EC-2 through EC-6 exercise `overrides_parse`/`overrides_apply` directly,
in-process; EC-7 through EC-9 pin the identical semantics through the
unilang `Kind::List(String, ',')` coercion and CLI dispatch boundary.

### Simple Co-Dependencies

Member of no [parameter group](../../../../../shader_chunks_query/tests/docs/cli/param_group/readme.md).
The only parameter in the CLI validated in two independent stages
(element shape, then bundle-relative identity) — EC-4/EC-5 fail at the
shape stage before any bundle exists; EC-3 fails at the identity stage
after the bundle is already built.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 9 |
| Real test functions | 9 |
| P0 (exit-code-affecting) | EC-3, EC-4, EC-5, EC-8, EC-9 |
| P1 (structural output) | EC-1, EC-2, EC-6, EC-7 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../command/cmd_001_render.md`](../command/cmd_001_render.md) | Sole command accepting `set` |
| [`../type/02_parameter_override.md`](../type/02_parameter_override.md) | Underlying override-element contract |
