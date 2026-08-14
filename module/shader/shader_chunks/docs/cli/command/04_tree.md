# Command :: 4. tree

### Description

Prints the dependency tree for one chunk, or — with no argument — a forest
of every chunk that nothing else depends on. Use it to understand
composition order before running `compose`.

-- **Parameters:** name
-- **Exit Codes:** 0 (success) | 1 (`name` given but does not resolve
   against `shader_chunks_core::CHUNKS`)
-- **Modes:** single tree (`name` given) | full forest (`name` omitted)

### Syntax
```bash
shader_chunks tree <name>
shader_chunks tree
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `name` | [`ChunkName`](../param/01_name.md) | `Varies` (omit for full forest) | No | Root chunk of the tree to show; omitted shows every root chunk |

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
```

### Notes
- "Root chunk" means a chunk nothing else in the bundled set depends on —
  currently `fbm3` and `fullscreen_triangle`.
- A dependency name that fails to resolve is skipped rather than causing a
  panic (defensive only — the bundled set is fixed and self-consistent, so
  this path is unreachable in practice today).
- Output format: [`tree_aligned`](../format/02_tree_aligned.md).

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.get`](02_get.md) | Flat detail for one node in the tree instead of the tree shape |
| 2 | [`.compose`](05_compose.md) | Actually produce the composed WGSL this tree previews the order of |

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
