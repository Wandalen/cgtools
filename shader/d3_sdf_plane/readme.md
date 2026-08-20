# d3_sdf_plane

Signed distance from a 3D point to an infinite plane with unit normal `n`
and offset `h` from the origin. The simplest possible 3D primitive —
ground planes, clipping planes, mirrors.

## Visualization

![d3_sdf_plane preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field —
no raymarcher or camera. The wrapper lifts each pixel into 3D as
`vec3f( p, 0.0 )` before calling `d3_sdf_plane( ·, vec3f( 0.0, 1.0, 0.0
), 0.0 )`; with a `z = 0` slice and this normal, `value` reduces exactly
to `p.y`. The lower half of the image (`p.y ≤ 0`) is solid black, since
every negative value clamps to `0`; the upper half is a clean linear
gradient to white — the signature look of a true plane's field, with no
curvature anywhere.

This demo is now wired in as a permanent `d3_sdf_plane_preview` export, so the chunk is directly previewable via `sch preview d3_sdf_plane` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d3_sdf_plane` |
| `description` | Signed distance from a 3D point to an infinite plane with unit normal n, offset h from the origin. |
| `tags` | `category:sdf, dim:3d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d3_sdf_plane(p: vec3f, n: vec3f, h: f32) -> f32`, `fn d3_sdf_plane_preview(p: vec2f, offset: f32, z_slice: f32) -> f32` |

## Nuances

- `n` must already be unit length — this chunk does not call `normalize`,
  to avoid paying that cost on every evaluation when `n` is a compile-time
  or per-draw constant.
- Positive `h` moves the plane along `-n`; the sign convention matches
  `dot(p,n) + h == 0` being the plane equation.
- Cheapest possible SDF primitive (one `dot` and one add) — good as a
  raymarch floor/backstop to guarantee ray termination.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d3_sdf_plane`, `sch tree d3_sdf_plane`)
- **Consumers:** none yet.
