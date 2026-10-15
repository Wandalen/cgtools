# Format :: F10. tree_mermaid

| Field | Value |
|-------|-------|
| ID | F10 |
| Output context | `tree shape::mermaid`, both with a `name` argument (single dependency tree) and without one (a forest — every root's reachable edges collected into one graph) |
| Trigger | `shape::mermaid` |
| Structure | A Mermaid `graph TD` flowchart: one `parent --> child` edge line per `depends_on` relationship (or its reverse, under `reverse::1`), plus one bare node line for any root with no children, so it still appears in the rendered graph. Chunk names are plain identifiers, so node IDs need no quoting. |
| Rendering mechanism | The same `collect_edges` walk [`tree_dot`](09_tree_dot.md) uses over `roots`/`children_of`; `mermaid_render` formats the collected edges and childless roots as `-->` lines instead of DOT statements |
| Example | See below |

### Example

`tree fbm3 shape::mermaid` (dependency chain `fbm3 → value_noise → hash21`):

```text
graph TD
  fbm3 --> value_noise
  value_noise --> hash21
```

`tree fullscreen_triangle shape::mermaid` (a root with no dependencies —
still declared, with no edge to carry it):

```text
graph TD
  fullscreen_triangle
```

Paste either block directly into a Mermaid Live Editor, or a Markdown
viewer with Mermaid support (a ```` ```mermaid ```` fence) — no
post-processing needed.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.tree](../command/04_tree.md) | Single tree (with `name`) or forest (without), `shape::mermaid` |

---

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`name`](../param/01_name.md) | Selects a single tree's root; omitted selects the full forest |
| 2 | [`reverse`](../param/22_reverse.md) | Flips edge direction from dependencies to dependents |
| 3 | [`shape`](../param/24_shape.md) | `mermaid` selects this format |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
