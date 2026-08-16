# d2_sdf_circle

Signed distance from a 2D point to a circle of the given radius: negative
inside, zero on the boundary, positive outside. The primitive behind
disks, dots, planet bodies — anything round that needs a crisp,
resolution-independent edge.

## Visualization

![d2_sdf_circle preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field:
`d2_sdf_circle( p, 0.28 )`'s raw signed-distance value is written
straight to each pixel as `vec3f( value )`, clamped to `[0, 1]`, at the
harness's default `preview_scale = 8`. Solid black fills the disk's
interior (distance ≤ 0); brightness grows linearly outward, reaching
white at distance `1`. The black/white edge traces the disk's true
boundary directly — there is no banding or shading, just the raw field.

This demo is now wired in as a permanent `d2_sdf_circle_preview`
export, so the chunk is directly previewable via
`sch preview d2_sdf_circle` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d2_sdf_circle` |
| `description` | Signed distance from a 2D point to a circle of the given radius. |
| `tags` | `category:sdf, dim:2d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d2_sdf_circle(p: vec2f, radius: f32) -> f32`, `fn d2_sdf_circle_preview(p: vec2f, radius: f32) -> f32` |

## Nuances

- Exact euclidean distance, not an approximation — safe for raymarching
  steps, offsetting (`d - t` grows the circle by `t`), and boolean ops
  (`min` = union, `max` = intersection) without error accumulation.
- The circle is centered at the origin: translate by subtracting the
  center from `p` before the call, as the preview harness does.
- Threshold the result with `aa_step` for antialiased fills, or feed
  `max( d, 0.0 )` into `glow` for a halo hugging the outside of the disk.
- Part of the `d2_sdf_*` / `d3_sdf_*` / `sdf_op_*` family — see
  [shader/](../readme.md) for the full SDF catalog and naming rationale.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d2_sdf_circle`, `sch tree d2_sdf_circle`)
- **Consumers:** none yet.
