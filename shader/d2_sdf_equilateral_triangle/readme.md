# d2_sdf_equilateral_triangle

Signed distance from a 2D point to an equilateral triangle of the given
circumradius `r`, apex pointing up. Fold-symmetric closed form — no
per-edge branching beyond a single fold across the diagonal.

## Visualization

![d2_sdf_equilateral_triangle preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field:
`d2_sdf_equilateral_triangle( p, 0.28 )`'s raw signed-distance value is
written straight to each pixel as `vec3f( value )`, clamped to `[0, 1]`,
at `preview_scale = 8`. Solid black fills the triangle's interior
(apex up); brightness grows outward from each of its three straight
edges.

This demo is now wired in as a permanent
`d2_sdf_equilateral_triangle_preview` export, so the chunk is
directly previewable via `sch preview d2_sdf_equilateral_triangle` —
no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d2_sdf_equilateral_triangle` |
| `description` | Signed distance from a 2D point to an equilateral triangle of the given circumradius, apex up. |
| `tags` | `category:sdf, dim:2d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d2_sdf_equilateral_triangle(p: vec2f, r: f32) -> f32`, `fn d2_sdf_equilateral_triangle_preview(p: vec2f) -> f32` |

## Nuances

- Fixed orientation (apex up) — rotate `p` before the call for any other
  facing; there is no separate rotated variant in this collection.
- For an arbitrary (non-equilateral) triangle from three explicit
  vertices, `d2_sdf_segment` composed three ways with `sdf_op_union` gets
  the unsigned outline; a signed arbitrary-triangle SDF is Tier B
  (deferred, needs a winding-based inside test).
- `r` is the circumradius (center-to-apex distance), not the side length.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d2_sdf_equilateral_triangle`, `sch tree d2_sdf_equilateral_triangle`)
- **Consumers:** none yet.
