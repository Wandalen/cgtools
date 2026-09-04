# Format :: F05. expanded

| Field | Value |
|-------|-------|
| ID | F05 |
| Output context | `list`/`get` under `format::expanded` — `.get`'s default (fields per `fields::` projection) |
| Trigger | `format::expanded` on either query command; the default detail view for `get` |
| Structure | One `-[ RECORD N ]` banner per chunk, followed by one `field \| value` line per projected field, fields in projection order, field-name column padded to the widest name |
| Rendering mechanism | `data_fmt`'s `RowBuilder` → `build_view()` → `ExpandedFormatter::new()` → `Format::format()` — the postgres `\x`-style record layout |
| Example | See below |

### Example

`get hash21` (default
`fields::name,description,stage,tags,depends_on,exports`):

```text
-[ RECORD 1 ]
name        | hash21
description | Single-value hash of a 2D point into [0, 1).
stage       | (none)
tags        | category:hash
depends_on  | (none)
exports     | fn hash21(p: vec2f) -> f32
```

Multiple selected chunks render as `RECORD 1`, `RECORD 2`, … in result
order. Unlike `json`/`yaml`, field order here is *stable* — exactly the
`fields::` projection order — which is why the equality tests pin on this
format.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.get](../command/02_get.md) | Default format |
| 2 | [.list](../command/01_list.md) | Via `format::expanded` |

---

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`format`](../param/15_format.md) | Selects this rendering |
| 2 | [`fields`](../param/13_fields.md) | Field rows and their order |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
