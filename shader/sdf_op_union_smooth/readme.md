# sdf_op_union_smooth

Smoothly blended union of two signed distances: like `sdf_op_union`, but
the seam where the two shapes meet is a smooth fillet of radius `k`
instead of a sharp crease. The classic "metaball" blend.

## Visualization

![sdf_op_union_smooth preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field —
the same `d2_sdf_circle` (radius `0.22`, centered at `x = -0.13`) /
`d2_sdf_box` (half-extents `0.16`, centered at `x = 0.15`) input pair
as `sdf_op_union`'s preview, composed with `sdf_op_union_smooth( ·, ·,
0.08 )` instead. Raw result written straight to `vec3f( value )`,
clamped to `[0, 1]`, at `preview_scale = 8`. Compare directly to
`sdf_op_union`'s field with the same shapes — the join here bulges
outward into a smooth waist instead of a sharp crease.

This demo is now wired in as a permanent `sdf_op_union_smooth_preview` export, so the chunk is directly previewable via `sch preview sdf_op_union_smooth` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `sdf_op_union_smooth` |
| `description` | Smoothly blended union of two signed distances with blend radius k. |
| `tags` | `category:sdf, technique:operator` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | `d2_sdf_circle`, `d2_sdf_box` |
| `export` | `fn sdf_op_union_smooth(d1: f32, d2: f32, k: f32) -> f32`, `fn sdf_op_union_smooth_preview(p: vec2f) -> f32` |

## Nuances

- `k -> 0` converges to plain `sdf_op_union`; larger `k` widens and
  softens the blend region, but also fattens the combined silhouette
  slightly beyond either shape's original extent (the `- k*h*(1-h)` term).
- `h` is a smoothstep-like blend factor derived from how close `d1` and
  `d2` are to each other, relative to `k` — where the two distances are
  far apart, `h` saturates to `0` or `1` and the result matches whichever
  shape is nearer, same as the sharp union.
- Not associative — blending three shapes pairwise gives a different
  result depending on grouping order, unlike the sharp operators.

## Relatives

- **Depends on:** [`d2_sdf_circle`](../d2_sdf_circle/readme.md), [`d2_sdf_box`](../d2_sdf_box/readme.md).
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get sdf_op_union_smooth`, `sch tree sdf_op_union_smooth`)
- **Consumers:** none yet.
