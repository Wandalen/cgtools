# Command :: 4. tree

### Description

Prints the dependency tree for one chunk, or — with no argument — a forest
of every chunk that nothing else depends on. Use it to understand
composition order before running `compose`. With `reverse::1`, the walk
direction flips: shows what (transitively) depends on the given chunk
instead of what it depends on.

-- **Parameters:** name, reverse, shape
-- **Exit Codes:** 0 (success) | 1 (`name` given but does not resolve
   against `shader_chunks_core::CHUNKS`, or `shape` is not one of
   `aligned`/`dot`/`mermaid`)
-- **Modes:** single tree (`name` given) | full forest (`name` omitted) |
   forward (`reverse::0`, default) | reverse/dependents (`reverse::1`) |
   aligned text (`shape::aligned`, default) | Graphviz (`shape::dot`) |
   Mermaid (`shape::mermaid`)

### Syntax
```bash
shader_chunks tree <name>
shader_chunks tree
shader_chunks tree <name> reverse::1
shader_chunks tree reverse::1
shader_chunks tree <name> shape::dot
shader_chunks tree <name> shape::mermaid
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `name` | [`ChunkName`](../param/01_name.md) | `Varies` (omit for full forest) | No | Root chunk of the tree to show; omitted shows every root chunk |
| `reverse` | [`Switch`](../type/07_switch.md) | `false` | No | Walk dependents instead of dependencies |
| `shape` | [`TreeFormat`](../type/11_tree_format.md) | `aligned` | No | Rendering shape: `aligned` (indented text), `dot` (Graphviz digraph), `mermaid` (Mermaid `graph TD`) |

### Examples
```bash
shader_chunks tree fbm3
# └── fbm3             category:noise, technique:fractal
#     └── value_noise  category:noise
#         └── hash21   category:hash

shader_chunks tree
# forest of every root chunk (42 in the current registry) -- e.g.:
# └── fullscreen_triangle  category:vertex
#
# └── hash33  category:hash
# ...

shader_chunks tree hash21 reverse::1
# └── hash21                   category:hash
#     └── value_noise          category:noise
#         └── fbm3             category:noise, technique:fractal
#             └── domain_warp  category:noise, technique:warp

shader_chunks tree fbm3 shape::dot
# digraph chunks
# {
#   "fbm3" -> "value_noise";
#   "value_noise" -> "hash21";
# }

shader_chunks tree fbm3 shape::mermaid
# graph TD
#   fbm3 --> value_noise
#   value_noise --> hash21
```

### Notes
- "Root chunk" means a chunk nothing else in the bundled set depends on —
  e.g. `domain_warp` and `fullscreen_triangle` (42 such chunks in the
  current registry; `fbm3` is not one of them — `domain_warp` itself
  depends on it, per the `reverse::1` example above). In `reverse::1` mode
  the forest roots flip to "leaf" chunks — those with an empty
  `depends_on` — since a reverse walk has no forward-root concept of its
  own to start from; see [`22_reverse.md`](../param/22_reverse.md).
- A dependency name that fails to resolve is skipped rather than causing a
  panic (defensive only — the bundled set is fixed and self-consistent, so
  this path is unreachable in practice today).
- Output format: selectable via `shape::` —
  [`tree_aligned`](../format/02_tree_aligned.md) (default, for a human to
  read), [`tree_dot`](../format/09_tree_dot.md) (Graphviz), or
  [`tree_mermaid`](../format/10_tree_mermaid.md) (Mermaid) — all three
  render the same walk, identical in both directions; only the edge
  direction being walked changes, never the shape's own structure.

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.get`](02_get.md) | Flat detail for one node in the tree instead of the tree shape |
| 2 | [`.compose`](../../../../shader_chunks_compose/docs/cli/command/01_compose.md) | Actually produce the composed WGSL this tree previews the order of |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*

---

**Category:** chunk
**Complexity:** 1
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low
