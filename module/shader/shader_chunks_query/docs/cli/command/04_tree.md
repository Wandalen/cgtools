# Command :: 4. tree

### Description

Prints the dependency tree for one chunk, or — with no argument — a forest
of every chunk that nothing else depends on. Use it to understand
composition order before running `compose`. With `reverse::1`, the walk
direction flips: shows what (transitively) depends on the given chunk
instead of what it depends on.

-- **Parameters:** name, reverse
-- **Exit Codes:** 0 (success) | 1 (`name` given but does not resolve
   against `shader_chunks_core::CHUNKS`)
-- **Modes:** single tree (`name` given) | full forest (`name` omitted) |
   forward (`reverse::0`, default) | reverse/dependents (`reverse::1`)

### Syntax
```bash
shader_chunks tree <name>
shader_chunks tree
shader_chunks tree <name> reverse::1
shader_chunks tree reverse::1
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `name` | [`ChunkName`](../param/01_name.md) | `Varies` (omit for full forest) | No | Root chunk of the tree to show; omitted shows every root chunk |
| `reverse` | [`Switch`](../type/07_switch.md) | `false` | No | Walk dependents instead of dependencies |

### Examples
```bash
shader_chunks tree fbm3
# fbm3           category:noise, technique:fractal
# └─ value_noise category:noise
#    └─ hash21   category:hash

shader_chunks tree
# fbm3                 category:noise, technique:fractal
# └─ value_noise       category:noise
#    └─ hash21         category:hash
# fullscreen_triangle  category:vertex

shader_chunks tree hash21 reverse::1
# hash21             category:hash
# └─ value_noise     category:noise
#    └─ fbm3         category:noise, technique:fractal
```

### Notes
- "Root chunk" means a chunk nothing else in the bundled set depends on —
  currently `fbm3` and `fullscreen_triangle`. In `reverse::1` mode the
  forest roots flip to "leaf" chunks — those with an empty `depends_on` —
  since a reverse walk has no forward-root concept of its own to start
  from; see [`22_reverse.md`](../param/22_reverse.md).
- A dependency name that fails to resolve is skipped rather than causing a
  panic (defensive only — the bundled set is fixed and self-consistent, so
  this path is unreachable in practice today).
- Output format: [`tree_aligned`](../format/02_tree_aligned.md) — identical
  in both directions, only the edge direction being walked changes.

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
