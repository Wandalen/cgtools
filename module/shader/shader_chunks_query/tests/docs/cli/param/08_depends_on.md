# Parameter Test :: depends_on

Source: [`../../../../docs/cli/param/08_depends_on.md`](../../../../docs/cli/param/08_depends_on.md)

### Edge Cases

| ID | Case | Real Test |
|----|------|-----------|
| EC-1 | Valid chunk — direct dependents selected | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_depends_on_selects_direct_dependents_and_transitive_widens` |
| EC-2 | Same chunk with `transitive::1` — closure widens the set | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_depends_on_selects_direct_dependents_and_transitive_widens` |
| EC-3 | Unknown chunk fails loudly (`CliError::UnknownChunk`) | `shader_chunks_query_core/tests/shader_chunks_query_core_test.rs::query_depends_on_unknown_chunk_fails_loudly` |

### Simple Co-Dependencies

Widened by [`transitive::`](09_transitive.md). Unlike
[`stage::`](07_stage.md)'s open selector, this value names a registry row
and is validated as one (EC-3).

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Edge cases | 3 |
| Real test functions | 2 |
| P0 (exit-code-affecting) | EC-3 |
| P1 (structural output) | EC-1, EC-2 |

### Cross-References

| File | Relationship |
|------|--------------|
| [`../type/01_chunk_name.md`](../type/01_chunk_name.md) | Underlying type contract |
| [`../param_group/01_filtering.md`](../param_group/01_filtering.md) | Interaction with `transitive::` |
