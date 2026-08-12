# Command :: 2. get

### Description

Prints full detail for one bundled shader chunk: name, description, stage,
tags, declared dependencies, and exported symbols. Use it once `list` has
told you which chunk name to inspect.

-- **Parameters:** name
-- **Exit Codes:** 0 (success) | 1 (`name` does not resolve against
   `shader_chunks_core::ALL_CHUNKS`)
-- **Modes:** (none)

### Syntax
```bash
shader_chunks get <name>
```

### Parameters

| Parameter | Type | Default | Required | Purpose |
|-----------|------|---------|----------|---------|
| `name` | [`ChunkName`](../param/01_name.md) | — | Yes | Which bundled chunk to show detail for |

### Examples
```bash
shader_chunks get hash21
# name: hash21
# description: Single-value hash of a 2D point into [0, 1).
# stage: None
# tags: category:hash
# depends_on: (none)
# exports:
#   fn hash21(p: vec2f) -> f32

shader_chunks get bogus_chunk
# unknown chunk: `bogus_chunk` (see `list` for valid names)
# (exit 1)
```

### Notes
- `stage` prints `None` for chunks with no declared pipeline stage (e.g.
  `hash21`, a pure helper function) — `Some("vertex")`-style values appear
  for chunks like `fullscreen_triangle` that do declare one.
- Never panics on an unknown name — reports `CliError::UnknownChunk` on
  stderr and exits 1.
- Output format: [`plain_text`](../format/03_plain_text.md).

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.list`](01_list.md) | Discover valid `name` values first |
| 2 | [`.tree`](04_tree.md) | See this chunk's dependency chain instead of its flat detail |
| 3 | [`.compose`](05_compose.md) | Combine this chunk with others into composed WGSL |

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*

---

**Category:** chunk
**Complexity:** 3
**API Requirement:** None
**Idempotent:** Yes
**Risk Level:** Low
