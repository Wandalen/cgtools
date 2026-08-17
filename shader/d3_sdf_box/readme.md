# d3_sdf_box

Signed distance from a 3D point to an axis-aligned box of the given
half-extents. The 3D counterpart of `d2_sdf_box` — crates, buildings,
bounding volumes.

## Visualization

![d3_sdf_box preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field —
no raymarcher or camera; the harness only samples a flat 2D plane. The
wrapper lifts each pixel into 3D as `vec3f( p, 0.0 )` (a slice through
the box's center on `z = 0`) before calling `d3_sdf_box( ·, vec3f(
0.28, 0.18, 0.22 ) )`, and the raw value is written straight to
`vec3f( value )`, clamped to `[0, 1]`, at `preview_scale = 8`. The slice
reproduces the box's `( 0.28, 0.18 )` xy cross-section exactly — solid
black interior, brightening outward from each edge.

This demo is now wired in as a permanent `d3_sdf_box_preview` export, so the chunk is directly previewable via `sch preview d3_sdf_box` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d3_sdf_box` |
| `description` | Signed distance from a 3D point to an axis-aligned box of the given half-extents. |
| `tags` | `category:sdf, dim:3d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d3_sdf_box(p: vec3f, half_extents: vec3f) -> f32`, `fn d3_sdf_box_preview(p: vec2f, half_extent_x: f32, half_extent_y: f32, half_extent_z: f32, z_slice: f32) -> f32` |

## Nuances

- Exact distance everywhere, including edges and corners — the
  `min(max(...),0.0)` inside-term and `length(max(...,0))` outside-term
  split is what makes both regions exact, same structure as `d2_sdf_box`.
- Centered and axis-aligned: rotate `p` by the inverse rotation for any
  other orientation.
- Feeds `d3_sdf_round_box` directly — see that chunk's `depends_on`.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** [`d3_sdf_round_box`](../d3_sdf_round_box/readme.md)
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d3_sdf_box`, `sch tree d3_sdf_box`)
- **Consumers:** none yet.
