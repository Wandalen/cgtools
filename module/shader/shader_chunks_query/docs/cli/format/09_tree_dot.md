# Format :: F09. tree_dot

| Field | Value |
|-------|-------|
| ID | F09 |
| Output context | `tree shape::dot`, both with a `name` argument (single dependency tree) and without one (a forest — every root's reachable edges collected into one graph) |
| Trigger | `shape::dot` |
| Structure | A Graphviz `digraph chunks { ... }` block: one `"parent" -> "child";` edge statement per `depends_on` relationship (or its reverse, under `reverse::1`), plus one bare `"name";` declaration for any root with no children, so it still appears in the rendered graph |
| Rendering mechanism | `collect_edges` walks the same `roots`/`children_of` structure [`tree_aligned`](02_tree_aligned.md) builds its `TreeNode`s from, collecting `(parent, child)` pairs once each (a diamond dependency's converging child is not re-expanded); `dot_render` formats the collected edges and childless roots |
| Example | See below |

### Example

`tree fbm3 shape::dot` (dependency chain `fbm3 → value_noise → hash21`):

```text
digraph chunks
{
  "fbm3" -> "value_noise";
  "value_noise" -> "hash21";
}
```

`tree fullscreen_triangle shape::dot` (a root with no dependencies —
still declared, with no edge to carry it):

```text
digraph chunks
{
  "fullscreen_triangle";
}
```

`tree hash21 reverse::1 shape::dot` (reverse walk — dependents instead
of dependencies, same edge-direction flip as `tree_aligned`):

```text
digraph chunks
{
  "hash21" -> "value_noise";
  "value_noise" -> "fbm3";
  "fbm3" -> "domain_warp";
}
```

Paste any of the above directly into Graphviz (`dot -Tpng`) or an online
Graphviz renderer — no post-processing needed.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.tree](../command/04_tree.md) | Single tree (with `name`) or forest (without), `shape::dot` |

---

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`name`](../param/01_name.md) | Selects a single tree's root; omitted selects the full forest |
| 2 | [`reverse`](../param/22_reverse.md) | Flips edge direction from dependencies to dependents |
| 3 | [`shape`](../param/24_shape.md) | `dot` selects this format |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
