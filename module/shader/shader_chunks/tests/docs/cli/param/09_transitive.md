# Parameter Test :: transitive

Source: [`../../../../docs/cli/param/09_transitive.md`](../../../../docs/cli/param/09_transitive.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | `false` (default): only direct dependents of `depends_on::`'s chunk | `shader_chunks_test.rs::query_depends_on_selects_direct_dependents_and_transitive_widens` |
| EC-2 | `true`: transitive closure — the chain's far end joins the result | `shader_chunks_test.rs::query_depends_on_selects_direct_dependents_and_transitive_widens` |
| EC-3 | `true` on `compose`: one root name pulls its whole chain, byte-identical to the explicit full set | `shader_chunks_test.rs::compose_chunks_transitive_closure_equals_the_explicit_full_set`; `cli_subprocess_test.rs::compose_single_name_with_transitive_pulls_the_full_dependency_chain` |
| EC-4 | `false` (default) on `compose`: named set must be dependency-complete — missing dependency exits 1 loudly | `shader_chunks_test.rs::compose_chunks_reports_missing_dependency_error_when_hash21_is_omitted`; `cli_subprocess_test.rs::compose_missing_dependency_exits_non_zero_without_a_panic_backtrace` |
| EC-5 | `true` on `compose` with a bogus root: the closure walk's lookup fails as loudly as a directly-named chunk | `shader_chunks_test.rs::compose_chunks_transitive_reports_unknown_chunk_error_for_bogus_name` |

### Simple Co-Dependencies

Per-command meaning: on `.list`/`.get` a pure modifier of
[`depends_on::`](08_depends_on.md) — a no-op without it; on `.compose` a
standalone widener of the positional `names` set itself.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 5 |
| Real test functions | 6 |
| P1 (structural output) | EC-1, EC-2, EC-3 |
| P0 (exit-code-affecting) | EC-4, EC-5 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/07_switch.md`](../type/07_switch.md) | Underlying boolean coercion contract |
| [`../param_group/01_filtering.md`](../param_group/01_filtering.md) | The modifier pair |
| [`../command/cmd_005_compose.md`](../command/cmd_005_compose.md) | `compose` closure usage context |
