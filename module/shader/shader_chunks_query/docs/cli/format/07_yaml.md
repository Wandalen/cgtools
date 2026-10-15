# Format :: F07. yaml

| Field | Value |
|-------|-------|
| ID | F07 |
| Output context | `list`/`get` under `format::yaml` (keys per `fields::` projection) |
| Trigger | `format::yaml` on either query command |
| Structure | YAML sequence of mappings, one `- ` entry per chunk, one `field: value` line per projected field — values as rendered strings (`(none)` placeholders included) |
| Rendering mechanism | `data_fmt`'s `RowBuilder` → `build_view()` → `YamlFormatter::new()` → `Format::format()` (serde_yaml_ng underneath) |
| Example | See below |

### Example

`list format::yaml fields::name,stage`:

```yaml
- name: hash21
  stage: (none)
- name: value_noise
  stage: (none)
- name: fbm3
  stage: (none)
- name: fullscreen_triangle
  stage: vertex
```

**Key order within a mapping is NOT guaranteed** — same hash-based row
model as [`json`](06_json.md); parse, don't string-match. Sequence order
*is* guaranteed: it is the sorted, paged result order.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.list](../command/01_list.md) | Via `format::yaml` |
| 2 | [.get](../command/02_get.md) | Via `format::yaml` |

---

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`format`](../param/15_format.md) | Selects this rendering |
| 2 | [`fields`](../param/13_fields.md) | Mapping keys |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
