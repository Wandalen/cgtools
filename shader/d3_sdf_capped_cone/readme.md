# d3_sdf_capped_cone

Signed distance from a 3D point to a capped (frustum) cone of half-height
`h` between bottom radius `r1` and top radius `r2`, axis along y.

## Visualization

![d3_sdf_capped_cone preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field —
no raymarcher or camera. The wrapper lifts each pixel into 3D as
`vec3f( p.x, p.y, 0.0 )` before calling `d3_sdf_capped_cone( ·, 0.22,
0.22, 0.08 )`; since the cone's axis runs along y with `p.xz` as the
radial plane, this `z = 0` slice is the *axial* profile — a trapezoid
tapering from radius `0.22` at the bottom to `0.08` at the top — not a
face-on circular cap. Raw value written straight to `vec3f( value )`,
clamped to `[0, 1]`, at `preview_scale = 8`.

This demo is now wired in as a permanent `d3_sdf_capped_cone_preview` export, so the chunk is directly previewable via `sch preview d3_sdf_capped_cone` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d3_sdf_capped_cone` |
| `description` | Signed distance from a 3D point to a capped cone of half-height h between radii r1 (bottom) and r2 (top), axis along y. |
| `tags` | `category:sdf, dim:3d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d3_sdf_capped_cone(p: vec3f, h: f32, r1: f32, r2: f32) -> f32`, `fn d3_sdf_capped_cone_preview(p: vec2f, half_height: f32, radius_bottom: f32, radius_top: f32, z_slice: f32) -> f32` |

## Nuances

- `r1 == r2` degenerates to `d3_sdf_capped_cylinder`; `r2 == 0` gives a
  sharp-tipped cone with a flat bottom only.
- The `ca`/`cb` split measures distance to the cap plane and the slanted
  side separately, then the sign flip picks whichever is the true nearest
  surface (or negates for points inside both regions).
- Costlier than `d3_sdf_capped_cylinder` (more ops) — prefer the cylinder
  chunk when `r1 == r2` is known statically.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d3_sdf_capped_cone`, `sch tree d3_sdf_capped_cone`)
- **Consumers:** none yet.
