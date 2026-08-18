# Feature: Terminal Backend Adapter

`adapters::TerminalBackend` implements the core `Backend` trait to render a command stream as an
ANSI-truecolor character-cell grid, behind the `adapter-terminal` feature.

### Scope

- **Purpose**: Let a command stream produce a terminal-renderable (ANSI truecolor) preview with
  no external dependencies.
- **Responsibility**: Cross-reference the terminal adapter's source and its documented
  simplifications relative to a full sub-pixel rasterizer.
- **In Scope**: Cell-grid rendering for every command family (paths, text, sprites, meshes,
  batches, groups) and the ANSI SGR truecolor encoding `output()` produces.
- **Out of Scope**: SVG, WebGL2, None, WebGPU, and Native adapters (see
  [001](001_svg_backend_adapter.md), [002](002_webgl2_backend_adapter.md),
  [004](004_none_backend_adapter.md), [005](005_webgpu_backend_adapter.md),
  [006](006_native_backend_adapter.md)).

### Design

World-space commands downsample onto a fixed character-cell grid: `cols`/`rows` are computed from
`RenderConfig`'s pixel `width`/`height` via ceiling division by `CELL_PX_WIDTH`×`CELL_PX_HEIGHT`
(16×32 world units per cell, chosen to approximate a monospace glyph's roughly 1:2 aspect ratio),
so a partially-covered trailing cell is never dropped. `resize` reallocates the grid at the new
dimensions. Each cell holds an owned `TerminalCell { glyph, bg, fg }`; `submit` resets every cell
to `RenderConfig::background` before dispatching the command batch, matching the other backends'
per-submit-clear convention.

World Y-up is converted to the grid's row-down addressing per positioned element (mirroring the
SVG adapter's own Y-up→Y-down handling — see
[invariant/001](../invariant/001_y_up_coordinate_system.md)); `world_to_cell` performs the flip
and bounds-checks the result, silently dropping anything outside the grid rather than panicking.

Single-draw commands (`Clear`, `Sprite`/`ScreenSpaceSprite`, `Mesh`) each resolve to one painted
cell at their transform's position; `Mesh` only paints when its fill is `FillRef::Solid` (gradient
and pattern fills are accepted but currently paint nothing). Paths (`BeginPath`..`EndPath`)
accumulate resolved cell coordinates per sub-path and connect consecutive points with true
Bresenham line rasterization (the symmetric integer variant — `line_cells(a, b)` always equals
`line_cells(b, a)` reversed); curve commands (`QuadTo`/`CubicTo`/`ArcTo`) flatten into a fixed
16 straight segments (`CURVE_SEGMENTS`) before rasterization — `ArcTo` derives its center via
the SVG 1.1 Appendix F.6.5 endpoint-to-center parameterization, falling back to a single
endpoint point for degenerate (zero-radius or coincident-endpoint) arcs. Text (`BeginText`/`Char`/`EndText`) is the one command family the terminal medium
represents natively — glyphs place directly into cells, with both horizontal
(left/center/right) and vertical (top/center/bottom) anchor support; vertical anchor nudges
the resolved row by a fraction of one cell height (0/½/1×`CELL_PX_HEIGHT`), the same top
hanging/center/bottom baseline split SVG expresses via `dominant-baseline`. Batches (`Create`/`Bind`/`Add`/
`Set`/`Remove`/`Draw`/`Delete`, both sprite and mesh) compose each instance's transform through the
batch's own parent transform. Groups (`BeginGroup`/`EndGroup`) push/pop a `Transform` stack folded
via `Transform::to_mat3`, right-to-left (`group_stack.iter().rev()`); group-level clip masks and
visual effects are accepted but not honored.

`output()` returns `Output::String`, encoding the grid as 24-bit ANSI SGR truecolor escape
sequences — one `\x1b[48;2;r;g;bm` background-color run per painted-background cell, one
`\x1b[38;2;r;g;bm{glyph}` foreground run per glyph cell, `\x1b[0m` reset and `\n` at the end of
each row.

Given the coarse cell resolution, no gradient/pattern fills, only a single blend mode
(`capabilities().supported_blend_modes` is `&[BlendMode::Normal]` — source-over Porter-Duff
alpha compositing on straight RGBA via `composite_over`; other variants fall back to it), and
no clip-mask/effect support, this adapter's status is tracked as partial (⚠️), matching the same
convention used for the WebGL2 and SVG adapters' own known gaps.
Remaining gaps are tracked in `roadmap.md`'s "terminal adapter gaps" section, not here — per this
crate's documentation split, forward-looking scope belongs in `roadmap.md`.

### Patterns

| File | Relationship |
|------|--------------|
| [pattern/001_ports_and_adapters_backend_architecture.md](../pattern/001_ports_and_adapters_backend_architecture.md) | This adapter is a third `Backend` implementation within the crate's hexagonal architecture |

### Sources

| File | Relationship |
|------|--------------|
| `src/adapters/terminal.rs` | Full `TerminalBackend` implementation — cell grid, command dispatch, ANSI SGR encoding |

### Tests

| File | Relationship |
|------|--------------|
| `tests/terminal_backend_test.rs` | Behavior tests drive `TerminalBackend` through its public surface (`Backend` trait plus `#[doc(hidden)]`-exported cell accessors), asserting on painted cell content and on exact ANSI byte output from `output()`. Covers grid dimensions, single-draw commands, missing-asset errors, path rasterization, text anchoring, group transforms, batch lifecycle (including out-of-bounds `Set`/no-op `Remove`/missing-id `Draw`), `assets_load` batch-clearing, and `resize` |
| `tests/backend_test.rs` | `Backend` trait contract exercised generically (not terminal-specific) |
