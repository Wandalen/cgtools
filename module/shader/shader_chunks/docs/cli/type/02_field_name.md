# Type :: 2. FieldName

**Purpose:** Identifies one queryable field of a chunk record — the unit
of column projection for the query engine behind `list`/`get`.

**Fundamental Type:** `String` (element of unilang
`Kind::List(String, ',')`). No Rust newtype exists — the closed valid set
is the `pub const QUERY_FIELDS : &[&str]` slice in `src/lib.rs`, and
validation is a slice-membership check at query time. Documented as a
semantic type because it carries a real closed-set constraint and a real
validate step.

**Constraints:**
- Must be one of exactly 7 values: `name`, `description`, `stage`,
  `tags`, `depends_on`, `exports`, `source`
- Case-sensitive, no aliases, no fuzzy matching
- Duplicates and arbitrary order are allowed — projection renders columns
  as given

**Parsing:** `query_chunks` (`src/lib.rs`) checks every requested field
against `QUERY_FIELDS` before any rendering. No match →
`CliError::UnknownField(field)`, reported as `` unknown field: `<field>`
(valid fields: name, description, stage, tags, depends_on, exports,
source) `` on stderr with a non-zero exit — never a silently empty column.

**Methods:**
- `field_value(chunk, field) -> String` (`src/lib.rs`) — renders the
  field's value for one chunk; `stage`/`depends_on`/`exports` render
  `(none)` when absent, `source` is the raw WGSL body
- `is_valid(field) -> bool` — conceptually
  `QUERY_FIELDS.contains(&field)`; realized inside `query_chunks`'s
  up-front validation loop

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `fields::` |
| 2 | [`.get`](../command/02_get.md) | `fields::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`fields`](../param/13_fields.md) | 2 |
