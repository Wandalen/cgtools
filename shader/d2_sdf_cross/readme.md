# d2_sdf_cross

Signed distance from a 2D point to a plus/cross shape of the given
half-extents and corner radius `r`. Two overlapping rectangles, folded
into a single edge test by symmetry.

## Visualization

![d2_sdf_cross preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field:
`d2_sdf_cross( p, vec2f( 0.28, 0.09 ), 0.02 )`'s raw signed-distance
value is written straight to each pixel as `vec3f( value )`, clamped to
`[0, 1]`, at `preview_scale = 8`. Solid black fills the plus-shaped
interior (12 corners, each rounded by `r`); brightness grows outward
from whichever of the two arms is nearest.

This demo is now wired in as a permanent `d2_sdf_cross_preview`
export, so the chunk is directly previewable via
`sch preview d2_sdf_cross` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d2_sdf_cross` |
| `description` | Signed distance from a 2D point to a plus/cross shape of the given half-extents and corner radius r. |
| `tags` | `category:sdf, dim:2d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d2_sdf_cross(p: vec2f, half_extents: vec2f, r: f32) -> f32`, `fn d2_sdf_cross_preview(p: vec2f, half_extent_x: f32, half_extent_y: f32, corner_radius: f32) -> f32` |

## Nuances

- `half_extents = ( long_arm, short_arm )` — the shape is the union of a
  `( long_arm, short_arm )` box and a `( short_arm, long_arm )` box, folded
  to one quadrant via the diagonal swap when `p.y > p.x`.
- `r = 0` gives sharp inner/outer corners; `r > 0` rounds all of them
  uniformly (both the outer tips and the inner reentrant corners).
- Useful directly as a UI/HUD crosshair or health-pack marker primitive.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d2_sdf_cross`, `sch tree d2_sdf_cross`)
- **Consumers:** none yet.
