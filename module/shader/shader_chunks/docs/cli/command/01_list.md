# Command :: 1. list

### Description

Prints a table of every bundled shader chunk: name, description, tags, and
declared dependencies. Use it to discover the exact chunk names accepted by
`get`, `tree`, and `compose`.

-- **Parameters:** (none)
-- **Exit Codes:** 0 (success) | 2 (render error — internal, `data_fmt` table
   formatting failed)
-- **Modes:** (none)

### Syntax
```bash
shader_chunks list
```

### Parameters

*(none)*

### Examples
```bash
shader_chunks list
# name                 description                                    tags                                depends_on
# hash21               Single-value hash of a 2D point into [0, 1).   category:hash                       (none)
# value_noise          Smooth 2D value noise in [0, 1).                category:noise                      hash21
# fbm3                 3-octave fractal Brownian motion of value_noise. category:noise, technique:fractal   value_noise
# fullscreen_triangle  Full-screen triangle vertex shader.             category:vertex                     (none)
```

### Notes
- Read-only — never modifies chunk data, only reads and renders the chunks
  the `shader_chunks_core::CHUNKS` table bundles.
- Output format: [`table_plain`](../format/01_table_plain.md).

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.get`](02_get.md) | Drill into one row's full detail |
| 2 | [`.tags`](03_tags.md) | Alternative view, grouped by tag instead of by chunk |
| 3 | [`.tree`](04_tree.md) | Alternative view, grouped by dependency relationship |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*

---

**Category:** chunk
**Complexity:** 0
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low
