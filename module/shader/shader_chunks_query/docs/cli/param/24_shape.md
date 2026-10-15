# Parameter :: 24. shape

- **Fundamental Type:** [`TreeFormat`](../type/11_tree_format.md) (unilang
  `Kind::String`, parsed manually — the project's established
  Kind::Enum-avoidance convention, same as [`format`](15_format.md))
- **Constraints:** Exactly 3 lowercase spellings accepted: `aligned`,
  `dot`, `mermaid`; closed set — no aliases, no case-insensitivity
- **Default:** `aligned` (the original indented-text rendering)
- **Purpose:** Modifier for `.tree` only — no other command accepts it.
  Selects the tree's rendering shape: the original indented text, or a
  paste-able Graphviz `digraph` / Mermaid `graph TD` of the same
  roots/edges, for feeding into a graph-rendering tool instead of a
  terminal.

### Examples
```bash
# Valid values
tree fbm3 shape::dot       # digraph chunks { "fbm3" -> "value_noise"; "value_noise" -> "hash21"; }
tree fbm3 shape::mermaid   # graph TD / fbm3 --> value_noise / value_noise --> hash21
tree fbm3                  # shape omitted -- same as shape::aligned

# Invalid values (rejected with error)
tree fbm3 shape::svg       # Execution error: invalid `shape` value: `svg` (allowed: aligned, dot, mermaid)
```

### Notes
- `dot` and `mermaid` walk the exact same `roots`/`children_of` structure
  [`aligned`](../format/02_tree_aligned.md) does — same chunk selection,
  same `reverse::` direction flip — only the rendering syntax differs;
  see [`09_tree_dot.md`](../format/09_tree_dot.md) and
  [`10_tree_mermaid.md`](../format/10_tree_mermaid.md).
- A root with no children (e.g. `fullscreen_triangle`, or any other chunk
  nothing depends on and that depends on nothing itself) still gets a
  bare node declaration in `dot`/`mermaid` output — `aligned` shows every
  root unconditionally too, so both graph formats preserve that
  guarantee rather than silently dropping isolated roots.
- Member of no parameter group — like [`reverse`](22_reverse.md), `shape`
  is `.tree`-only and shares no co-occurrence group with the shared
  `.list`/`.get` surface.

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.tree](../command/04_tree.md) | `aligned` | Selects indented text, DOT, or Mermaid rendering |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| [TreeFormat](../type/11_tree_format.md) | String (enum-spelled) | `enum { Aligned, Dot, Mermaid }` | Closed 3-value set |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
