# Command :: 3. tags

### Description

Prints a table of every distinct `group:tag` pair used across the bundled
chunks and which chunk(s) carry it. Use it to discover chunks by category
or technique rather than by name.

-- **Parameters:** (none)
-- **Exit Codes:** 0 (success) | 2 (render error — internal, `data_fmt`
   table formatting failed)
-- **Modes:** (none)

### Syntax
```bash
shader_chunks tags
```

### Parameters

*(none)*

### Examples
```bash
shader_chunks tags
# tag                  chunks
# category:hash        hash21, hash22, hash13, hash33
# category:noise       value_noise, fbm3, value_noise3, gradient_noise, voronoi, domain_warp
# technique:fractal    fbm3
# category:vertex      fullscreen_triangle
# ...                  (one row per distinct group:tag pair)
```

### Notes
- A chunk with multiple tags (e.g. `fbm3`, tagged both `category:noise` and
  `technique:fractal`) appears once per tag it carries — one row per
  `group:tag` pair, not one row per chunk.
- Every `tag` cell is a valid [`tag::`](../param/05_tag.md) selector for
  `list`/`get` — this command enumerates that selector vocabulary.
- Output format: [`table_plain`](../format/01_table_plain.md).

### Related Commands

| # | Command | Relationship |
|---|---------|--------------|
| 1 | [`.list`](01_list.md) | Filter by these tags via `tag::`; grouped by chunk instead of by tag |
| 2 | [`.get`](02_get.md) | Full detail for one of the chunks a tag row names |

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
