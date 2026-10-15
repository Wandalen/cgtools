# Type :: 11. TreeFormat

**Purpose:** Selects which of the 3 renderings `tree` produces for the
same walked roots/edges.

**Fundamental Type:** `enum TreeFormat { Aligned, Dot, Mermaid }` in
`shader_chunks_query_core/src/lib.rs` —
`#[derive(Debug, Clone, Copy, PartialEq, Eq)]`, a genuine Rust enum
(unlike the string-realized selector types), passed as `chunk_tree`'s
third argument.

**Constraints:**
- Exactly 3 lowercase spellings accepted: `aligned`, `dot`, `mermaid`
- Closed set — no aliases, no case-insensitivity

**Parsing:** `TreeFormat::from_str` (via Rust's `FromStr`). No match →
`QueryError::InvalidParam { param: "shape", .. }`, reported as `` invalid
`shape` value: `<value>` (allowed: aligned, dot, mermaid) `` on stderr
with a non-zero exit.

**Methods:**
- `as_str() -> &'static str` — canonical lowercase spelling; round-trips
  with `from_str`, and supplies the `[default: …]` text in help screens
- `from_str(s) -> Result<Self, QueryError>` — the parse above
- Dispatch: `chunk_tree` (`shader_chunks_query_core/src/lib.rs`) matches
  on the variant — `Aligned` reuses the existing `TreeFormatter`/
  `TreeNode` pipeline; `Dot`/`Mermaid` walk `roots`/`children_of` via the
  shared `collect_edges` helper and render through `dot_render`/
  `mermaid_render` (see [`../format/`](../format/readme.md))

---

### Referenced Commands

| # | Command | Via Parameter |
|---|---------|----------------|
| 1 | [`.tree`](../command/04_tree.md) | `shape::` (default `aligned`) |

---

### Referenced Parameters

| # | Parameter | Commands |
|---|-----------|----------|
| 1 | [`shape`](../param/24_shape.md) | 1 |
