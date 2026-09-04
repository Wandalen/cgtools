# Type :: 5. SortOrder

**Purpose:** Selects the direction of the active [`SortKey`](04_sort_key.md)
ordering.

**Fundamental Type:** `enum SortOrder { Asc, Desc }` in `shader_chunks_query_core/src/lib.rs` —
`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, carried inside
`QueryParams`.

**Constraints:**
- Exactly 2 lowercase spellings accepted: `asc`, `desc`
- Closed set — no aliases, no case-insensitivity

**Parsing:** `SortOrder::from_str`. No match →
`CliError::InvalidParam { param: "order", .. }`, reported as `` invalid
`order` value: `<value>` (allowed: asc, desc) `` on stderr with a
non-zero exit.

**Methods:**
- `as_str() -> &'static str` — canonical spelling, round-trips with
  `from_str`
- Semantics (`chunks_query`): `Desc` is an exact `reverse()` of the fully
  sorted sequence — applied after tie-breaking, and equally to
  `sort::input` (reversed selection order)

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.list`](../command/01_list.md) | `order::` |
| 2 | [`.get`](../command/02_get.md) | `order::` |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`order`](../param/17_order.md) | 2 |
