# Type Test :: ParameterOverride

Source: [`../../../../docs/cli/type/02_parameter_override.md`](../../../../docs/cli/type/02_parameter_override.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | Well-formed `property:value` pairs parse and, applied to a real bundle, replace the named parameters' default values | `shader_chunks_render/tests/render_cli_test.rs::set_override_replaces_the_named_parameters_default_value` |
| TC-2 | Missing `:` separator: `overrides_parse` rejects the token before any bundle is touched | `shader_chunks_render/tests/render_cli_test.rs::overrides_parse_rejects_a_token_missing_its_separator` |
| TC-3 | Non-finite (`inf`/`-inf`/`nan`) or non-numeric value side: `overrides_parse` rejects the token | `shader_chunks_render/tests/render_cli_test.rs::overrides_parse_rejects_a_non_finite_or_non_numeric_value` |
| TC-4 | Identity resolution failure: a syntactically valid pair whose property matches none of the bundle's declared parameters is rejected by `overrides_apply`, naming every valid property | `shader_chunks_render/tests/render_cli_test.rs::set_override_rejects_an_unknown_parameter_name` |
| TC-5 | Resolution order: applying two pairs naming the same property in sequence leaves the later value in place | `shader_chunks_render/tests/render_cli_test.rs::overrides_apply_lets_a_later_override_of_the_same_property_win` |

TC-1/TC-4/TC-5 exercise `overrides_apply`'s bundle-relative identity
resolution; TC-2/TC-3 exercise `overrides_parse`'s bundle-independent
shape validation alone — the two stages fail for structurally different
reasons and are pinned separately. Per-parameter edge-case detail
remains owned by [`../param/04_set.md`](../param/04_set.md).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 5 |
| Real test functions referenced | 5 |
| Positive semantics | TC-1, TC-5 |
| Invalid-input rejection | TC-2, TC-3, TC-4 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/04_set.md`](../param/04_set.md) | Usage: the `set::` override list |
| [`TagSelector`](../../../../../shader_chunks_query/tests/docs/cli/type/09_tag_selector.md) | Sibling `Kind::List(String, ',')` element type with the same colon-pair grammar, filter-purposed rather than value-assignment-purposed |
