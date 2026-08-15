# Format :: F01. table_plain

| Field | Value |
|-------|-------|
| ID | F01 |
| Output context | `list`/`get` under `format::table` (columns per `fields::` projection) and `tags` (fixed columns: tag, chunks) |
| Trigger | `format::table` — `.list`'s default — on the query commands; always on `tags` (its only format) |
| Structure | Optional heading line (`heading::`), header row of column names, a dashes separator row, one data row per entry (long cells wrap onto continuation lines), columns left-aligned and padded (capped by `width::`), no box-drawing borders |
| Rendering mechanism | `data_fmt`'s `RowBuilder` → `build_view()` → `TableFormatter::with_config(TableConfig::plain())` (plus `with_heading`/`with_max_column_width` when set) → `Format::format()` pipeline |
| Example | See below |

### Example

`list` output shape (default `fields::name,description,tags,depends_on`;
long description cells wrap onto continuation lines):

```text
name                 description                        tags                               depends_on
-------------------  ---------------------------------  ---------------------------------  -----------
hash21               Single-value hash of a 2D point    category:hash                      (none)
                     into [0, 1).
value_noise          Bilinear-interpolated value noise  category:noise                     hash21
                     sampled at a 2D point, in [0, 1).
fbm3                 Fixed 3-octave fractal Brownian    category:noise, technique:fractal  value_noise
                     motion built on value_noise, in
                     [0, 0.875].
fullscreen_triangle  Fullscreen-triangle vertex stage:  category:vertex                    (none)
                     3 vertices, no vertex buffer,
                     vertex_index alone picks the
                     corner.
```

`tags` output shape (one row per distinct `group:tag` pair):

```text
tag                chunks
-----------------  -------------------
category:hash      hash21, hash22, hash13, hash33
technique:fractal  fbm3
category:vertex    fullscreen_triangle
```

Exact column widths are computed at render time by `TableFormatter` from
the widest cell in each column — the alignment above is illustrative, not a
byte-exact contract. On the query commands, `fields::` selects the
columns and [`heading::`](../param/20_heading.md)/[`width::`](../param/21_width.md)
shape the frame.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.list](../command/01_list.md) | Default format; columns per `fields::` |
| 2 | [.get](../command/02_get.md) | Via `format::table`; columns per `fields::` |
| 3 | [.tags](../command/03_tags.md) | Always; 2 fixed columns: tag, chunks |
| 4 | [.tunables](../../../../shader_chunks_params/docs/cli/command/01_tunables.md) | When the chunk declares ≥1 parameter; columns: name, kind, type, range, source — a chunk with zero declared parameters prints a one-line message instead |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
