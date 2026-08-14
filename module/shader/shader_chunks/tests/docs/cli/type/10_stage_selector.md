# Type Test :: StageSelector

Source: [`../../../../docs/cli/type/10_stage_selector.md`](../../../../docs/cli/type/10_stage_selector.md)

### Test Case Index

| ID | Case | Real Test |
|----|------|-----------|
| TC-1 | `any` (default): no stage filtering | `shader_chunks_test.rs::query_stage_filter_selects_none_literal_and_any` |
| TC-2 | `none`: selects only chunks with no stage | `shader_chunks_test.rs::query_stage_filter_selects_none_literal_and_any` |
| TC-3 | Literal (e.g. `vertex`): exact stage match | `shader_chunks_test.rs::query_stage_filter_selects_none_literal_and_any` |
| TC-4 | Unmatched literal: empty output, exit 0 — open selector, unlike the closed enums | `shader_chunks_test.rs::query_stage_filter_selects_none_literal_and_any` |

`StageSelector` is an open selector by design: `any`/`none` are the only
reserved words; every other string is a literal stage name, so no input
is invalid and no rejection case exists.

### Test Coverage Summary

| Metric | Value |
|--------|-------|
| Type cases | 4 |
| Real test functions referenced | 1 (covers all four arms) |
| Reserved-word arms | TC-1, TC-2 |
| Literal arm | TC-3, TC-4 |

### Cross-References

| File | Relationship |
|------|----------------|
| [`../param/07_stage.md`](../param/07_stage.md) | `stage` parameter — the sole usage context |
