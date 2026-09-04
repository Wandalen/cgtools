# Type :: 4. SortKey

**Purpose:** Selects the field the query engine orders its result by,
before paging and rendering.

**Fundamental Type:** `enum SortKey { Input, Name, Stage, Description }`
in `shader_chunks_query_core/src/lib.rs` — `#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, carried
inside `QueryParams`.

**Constraints:**
- Exactly 4 lowercase spellings accepted: `input`, `name`, `stage`,
  `description`
- Closed set — no aliases, no case-insensitivity

**Parsing:** `SortKey::from_str`. No match →
`CliError::InvalidParam { param: "sort", .. }`, reported as `` invalid
`sort` value: `<value>` (allowed: input, name, stage, description) `` on
stderr with a non-zero exit.

**Methods:**
- `as_str() -> &'static str` — canonical spelling, round-trips with
  `from_str`
- Sort semantics (`chunks_query`): `Input` is a no-op (selection order
  kept); `Name` sorts by chunk name; `Stage` and `Description` sort by
  `(field, name)` tuples — the name tie-break makes both deterministic,
  and stage-less chunks (empty stage) sort first

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `sort::` |
| 2 | [`.get`](../command/02_get.md) | `sort::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`sort`](../param/16_sort.md) | 2 |
