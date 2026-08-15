# d3_sdf_ellipsoid

Signed distance *bound* (not exact) from a 3D point to an ellipsoid of
the given per-axis radii `r`. The standard, widely-used approximation —
exact ellipsoid distance has no closed form.

## Visualization

![d3_sdf_ellipsoid preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field —
no raymarcher or camera. The wrapper lifts each pixel into 3D as
`vec3f( p, 0.0 )` (a center slice on `z = 0`) before calling
`d3_sdf_ellipsoid( ·, vec3f( 0.32, 0.18, 0.24 ) )`, raw value written
straight to `vec3f( value )`, clamped to `[0, 1]`, at `preview_scale =
8`. The zero level-set on this slice is an exact axis-aligned ellipse
with semi-axes `( 0.32, 0.18 )` — per this chunk's own Nuances, the
boundary is always exact even though this chunk's off-surface distance
is only a bound, so brightness away from the boundary is approximate.

This demo is now wired in as a permanent `d3_sdf_ellipsoid_preview` export, so the chunk is directly previewable via `sch preview d3_sdf_ellipsoid` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d3_sdf_ellipsoid` |
| `description` | Signed distance bound from a 3D point to an ellipsoid of the given per-axis radii (not exact). |
| `tags` | `category:sdf, dim:3d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d3_sdf_ellipsoid(p: vec3f, r: vec3f) -> f32`, `fn d3_sdf_ellipsoid_preview(p: vec2f) -> f32` |

## Nuances

- **Not exact** — a raymarcher must under-step (multiply the returned
  value by a safety factor `< 1`) near-elongated ellipsoids to avoid
  overshooting; `r == vec3f(radius)` (a sphere) degenerates to the exact
  `d3_sdf_sphere` distance, where the bound happens to be tight.
- The zero-contour (the ellipsoid surface itself) is exact regardless —
  only off-surface distance magnitude is approximate, which matters for
  raymarch step size but not for a final `aa_step`/mask threshold.
- Cheaper than trying to solve the true quartic distance-to-ellipsoid
  problem, which has no simple closed form.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d3_sdf_ellipsoid`, `sch tree d3_sdf_ellipsoid`)
- **Consumers:** none yet.
