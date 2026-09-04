# Parameter :: 25. size

- **Fundamental Type:** String (unilang `Kind::String`), parsed by
  `size_parse` (`shader_chunks_render/src/lib.rs`)
- **Constraints:** Either `<n>` (square) or `<width>x<height>` —
  lowercase `x`, each side a plain integer ≥ 1; surrounding whitespace
  is tolerated (`" 32 x 16 "` parses). Zero sides, missing sides,
  negatives, fractions, uppercase `X`, and extra segments are all
  rejected with exit 1: `` invalid `size` value: `<raw>` (allowed: `<n>`
  or `<width>x<height>`, each side at least 1) ``.
- **Default:** `256` (a 256×256 frame)
- **Purpose:** Pixel dimensions of the written PNG — and, identically,
  the `resolution` uniform the bundle receives, so a chunk that reads
  `params.resolution` sees exactly the frame it is being drawn into.

### Examples
```bash
# Valid values
render fbm3                 # 256x256 (default)
render fbm3 size::512       # square shorthand
render fbm3 size::128x64    # explicit width x height

# Invalid values (rejected with error)
render fbm3 size::0         # zero side, exit 1
render fbm3 size::64x       # missing height side, exit 1
render fbm3 size::256X256   # uppercase X is not the separator, exit 1
render fbm3 size::1.5       # fractional pixels, exit 1
```

### Notes
- Declared `Kind::String`, not `Kind::Integer`, because the two-form
  grammar (`256` vs `128x64`) is not a number — the real validation
  lives in `size_parse`, unit-tested in both directions
  (`size_parse_accepts_square_and_explicit_forms`,
  `size_parse_rejects_zero_missing_and_junk_sides`).
- A bad `size::` fails before the chunk is even resolved — the routine
  checks target-count, then size, then time, and only then builds the
  bundle, so argument errors never depend on GPU or registry state.
- Widths whose row bytes are not a 256-byte multiple are handled by the
  readback's row-padding strip, engine-tested at `100x50`
  (`render_handles_widths_whose_row_bytes_need_padding`) — any size
  with both sides ≥ 1 is safe, not just powers of two.
- Member of no [parameter group](../../../../shader_chunks_query/docs/cli/param_group/readme.md).

---

### Referenced Commands

| # | Command | Default | Notes |
|---|---------|---------|-------|
| 1 | [.render](../command/01_render.md) | `256` | Also sets the bundle's `resolution` uniform |

---

### Referenced Type

| Type | Kind | Fundamental | Key Constraint |
|------|------|-------------|----------------|
| String | String | `String` | Two-form grammar `<n>` / `<w>x<h>`, each side ≥ 1 |

---

### Referenced User Stories

*(None — this project deliberately omits the `user_story/` collection at
this CLI's scale; see [`docs/cli/readme.md` § Scope
Decisions](../readme.md#scope-decisions).)*
