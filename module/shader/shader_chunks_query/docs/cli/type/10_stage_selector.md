# Type :: 10. StageSelector

**Purpose:** The `stage::` filter value — a three-way selector over the
chunk's optional declared pipeline stage.

**Fundamental Type:** `String` (unilang `Kind::String`). No wrapper type
— the three-way branch is a string comparison in `chunk_matches`
(`shader_chunks_query_core/src/lib.rs`).

**Constraints:**
- `any` — reserved: no stage filtering (the default)
- `none` — reserved: only chunks with *no* declared stage
- Any other string — a literal stage name, compared exactly against the
  chunk's `//@ stage:` metadata; an unmatched literal yields empty
  output with exit 0, never an error (open selector, unlike the closed
  enum types)

**Parsing:** None beyond the reserved-word branch — `chunk_matches`
checks `"any"` (skip), then `"none"` (`chunk.stage.is_none()`), else
`chunk.stage == Some(selector)`.

**Methods:**
- `selects(selector, stage: Option<&str>) -> bool` — conceptually the
  branch above; realized inline in `chunk_matches`
- Rendering counterpart: the `stage` field renders `(none)` for
  stage-less chunks (`field_value` in `shader_chunks_query_core/src/lib.rs`), matching the
  `none` selector word

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `stage::` |
| 2 | [`.get`](../command/02_get.md) | `stage::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`stage`](../param/07_stage.md) | 2 |
