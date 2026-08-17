# Parameter :: 13. fields

- **Fundamental Type:** list of [`FieldName`](../type/02_field_name.md)
  (unilang `Kind::List(String, ',')` — one `fields::` value,
  comma-separated)
- **Constraints:** Every element must be one of the 7 queryable fields —
  `name`, `description`, `stage`, `tags`, `depends_on`, `exports`,
  `source` (`QUERY_FIELDS` in `shader_chunks_query_core/src/lib.rs`); an unknown element is
  `CliError::UnknownField` listing the valid set, non-zero exit
- **Default:** `Varies` — `name,description,tags,depends_on` for `list`
  (overview columns), `name,description,stage,tags,depends_on,exports`
  for `get` (detail columns)
- **Purpose:** Projects which fields render, in which order — from a
  single-column name listing up to the full record including the raw
  WGSL body (`source`).

### Examples
```bash
# Valid values
list fields::name                       # one column
get hash21 fields::name,source          # name + raw WGSL body
list fields::stage,name sort::stage     # projection order = column order

# Invalid values (rejected with error)
list fields::bogus   # "unknown field: `bogus` (valid fields: name,
                      #  description, stage, tags, depends_on, exports, source)"
```

### Notes
- Ignored by `format::names` (always exactly the names) and by
  `count::1` (a number has no columns).
- `stage` renders `(none)` for stage-agnostic chunks; `depends_on` and
  `exports` render `(none)` when empty — cells are never blank.
- Member of the [projection](../param_group/02_projection.md) parameter
  group; the per-command default is one of the two ways `list` and `get`
  differ (the other being [`format::`](15_format.md)).

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.list](../command/01_list.md) | `name,description,tags,depends_on` | Overview columns |
| 2 | [.get](../command/02_get.md) | `name,description,stage,tags,depends_on,exports` | Detail columns |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [FieldName](../type/02_field_name.md) | String (list) | `Vec<String>` | Closed set of 7 (`QUERY_FIELDS`) |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
