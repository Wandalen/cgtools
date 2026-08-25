# Format :: F04. markdown

| Field | Value |
|-------|-------|
| ID | F04 |
| Output context | `list`/`get` under `format::markdown` (columns per `fields::` projection) |
| Trigger | `format::markdown` on either query command |
| Structure | Optional heading rule (`heading::`), then a GitHub-style pipe table: `\| col \| … \|` header row, `\|---\|` separator row, one pipe row per chunk; cell width capped by `width::` with `...` truncation |
| Rendering mechanism | `data_fmt`'s `RowBuilder` → `build_view()` → `TableFormatter::with_config(TableConfig::markdown())` (plus `with_heading`/`with_max_column_width` when set) → `Format::format()` |
| Example | See below |

### Example

`list format::markdown heading::Chunks width::30`:

```text
─── Chunks ─────────────────────────────────────────────────────────────
| name                | description                    | tags                           | depends_on  |
|---------------------|--------------------------------|--------------------------------|-------------|
| hash21              | Single-value hash of a 2D p... | category:hash                  | (none)      |
| value_noise         | Bilinear-interpolated value... | category:noise                 | hash21      |
| fbm3                | Fixed 3-octave fractal Brow... | category:noise, technique:f... | value_noise |
| fullscreen_triangle | Fullscreen-triangle vertex ... | category:vertex                | (none)      |
```

Paste-ready for markdown documents; the heading rule renders only when
`heading::` is set. Exact padding is computed at render time — the shape
(pipe table with separator row) is the contract, byte-exact alignment is
not.

---

### Referenced Commands

| # | Command | Notes |
|---|---------|-------|
| 1 | [.list](../command/01_list.md) | Via `format::markdown` |
| 2 | [.get](../command/02_get.md) | Via `format::markdown` |

---

### Referenced Parameters

| # | Parameter | Role |
|---|-----------|------|
| 1 | [`format`](../param/15_format.md) | Selects this rendering |
| 2 | [`heading`](../param/20_heading.md) | Optional heading rule above the table |
| 3 | [`width`](../param/21_width.md) | Cell truncation cap |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
