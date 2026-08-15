# d2_sdf_hexagon

Signed distance from a 2D point to a regular hexagon of the given
circumradius `r`, flat side up. Two axis folds reduce the six-edge shape
to a single edge test.

## Visualization

![d2_sdf_hexagon preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field:
`d2_sdf_hexagon( p, 0.26 )`'s raw signed-distance value is written
straight to each pixel as `vec3f( value )`, clamped to `[0, 1]`, at
`preview_scale = 8`. Solid black fills the hexagon's interior (flat
top/bottom); brightness grows outward from each of its six edges.

This demo is now wired in as a permanent `d2_sdf_hexagon_preview`
export, so the chunk is directly previewable via
`sch preview d2_sdf_hexagon` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d2_sdf_hexagon` |
| `description` | Signed distance from a 2D point to a regular hexagon of the given circumradius. |
| `tags` | `category:sdf, dim:2d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d2_sdf_hexagon(p: vec2f, r: f32) -> f32`, `fn d2_sdf_hexagon_preview(p: vec2f) -> f32` |

## Nuances

- Fixed orientation (flat top/bottom edges) — rotate `p` by 30° for a
  pointy-top hexagon.
- The two `k.xy`-projected folds are what collapse 6-way symmetry into a
  single half-plane clamp; the constant `k` encodes `cos`/`sin` of 30°.
- Natural fit for tile/grid work alongside `voronoi` — a hex-grid cell
  outline where voronoi gives the cell id.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d2_sdf_hexagon`, `sch tree d2_sdf_hexagon`)
- **Consumers:** none yet.
