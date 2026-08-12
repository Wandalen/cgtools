# Format :: F01. table_plain

| Field | Value |
|-------|-------|
| ID | F01 |
| Output context | `list` (columns: name, description, tags, depends_on) and `tags` (columns: tag, chunks) |
| Trigger | Always — `list` and `tags` have exactly one output format, unconditional on any parameter |
| Structure | Header row of column names, one data row per entry, columns left-aligned and padded to the widest cell in that column, no box-drawing borders |
| Rendering mechanism | `data_fmt`'s `RowBuilder` → `build_view()` → `TableFormatter::with_config(TableConfig::plain())` → `Format::format()` pipeline |
| Example | See below |

### Example

`list` output shape (4 bundled chunks):

```text
name                 description                                   tags                              depends_on
hash21               Single-value hash of a 2D point into [0, 1).  category:hash                     (none)
value_noise          Smooth 2D value noise in [0, 1).               category:noise                    hash21
fbm3                 3-octave fractal Brownian motion of value_noise. category:noise, technique:fractal  value_noise
fullscreen_triangle  Full-screen triangle vertex shader.            category:vertex                   (none)
```

`tags` output shape (one row per distinct `group:tag` pair):

```text
tag                  chunks
category:hash        hash21
category:noise       value_noise, fbm3
technique:fractal    fbm3
category:vertex      fullscreen_triangle
```

Exact column widths are computed at render time by `TableFormatter` from
the widest cell in each column — the alignment above is illustrative, not a
byte-exact contract.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.list](../command/01_list.md) | 4 columns: name, description, tags, depends_on |
| 2 | [.tags](../command/03_tags.md) | 2 columns: tag, chunks |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
