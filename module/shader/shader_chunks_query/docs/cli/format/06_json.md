# Format :: F06. json

| Field | Value |
|-------|-------|
| ID | F06 |
| Output context | `list`/`get` under `format::json` (keys per `fields::` projection) |
| Trigger | `format::json` on either query command |
| Structure | Pretty-printed JSON array, one object per chunk, one `"field": "value"` pair per projected field — every value a string, exactly as the table cell would render (`(none)` placeholders included) |
| Rendering mechanism | `data_fmt`'s `RowBuilder` → `build_view()` → `JsonFormatter::new()` → `Format::format()` (serde_json underneath) |
| Example | See below |

### Example

`list format::json fields::name,stage`:

```json
[
  {
    "name": "hash21",
    "stage": "(none)"
  },
  {
    "name": "value_noise",
    "stage": "(none)"
  },
  {
    "name": "fbm3",
    "stage": "(none)"
  },
  {
    "stage": "vertex",
    "name": "fullscreen_triangle"
  }
]
```

**Key order within an object is NOT guaranteed** (note the last record
above) — `data_fmt`'s row model is hash-based. Consumers must parse
(`jq`, serde), never string-match on key position. Array order *is*
guaranteed: it is the sorted, paged result order.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.list](../command/01_list.md) | Via `format::json` |
| 2 | [.get](../command/02_get.md) | Via `format::json` |

---

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`format`](../param/15_format.md) | Selects this rendering |
| 2 | [`fields`](../param/13_fields.md) | Object keys |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
