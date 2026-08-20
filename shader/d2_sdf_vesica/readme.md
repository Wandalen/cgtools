# d2_sdf_vesica

Signed distance from a 2D point to a vesica (lens/almond) shape: the
intersection of two circles of radius `r` whose centers are offset by
`d` along the x-axis. Eyes, leaves, mandorla motifs.

## Visualization

![d2_sdf_vesica preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field:
`d2_sdf_vesica( p, 0.3, 0.15 )`'s raw signed-distance value is written
straight to each pixel as `vec3f( value )`, clamped to `[0, 1]`, at
`preview_scale = 8`. Solid black fills the lens; brightness grows
outward, meeting at two sharp pointed cusps top and bottom where the
two source circles would have crossed.

This demo is now wired in as a permanent `d2_sdf_vesica_preview` export, so the chunk is directly previewable via `sch preview d2_sdf_vesica` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d2_sdf_vesica` |
| `description` | Signed distance from a 2D point to a vesica (lens) shape from two circles of radius r offset by d. |
| `tags` | `category:sdf, dim:2d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d2_sdf_vesica(p: vec2f, r: f32, d: f32) -> f32`, `fn d2_sdf_vesica_preview(p: vec2f, radius: f32, offset: f32) -> f32` |

## Nuances

- Requires `d < r` — `b = sqrt(r*r - d*d)` goes imaginary (NaN) otherwise;
  `d == 0` degenerates to a single circle of radius `r`.
- The cusps are exact (not rounded) — a sharper alternative to intersecting
  two `d2_sdf_circle` calls with `sdf_op_intersect`, which would also work
  but costs two evaluations instead of one closed form.
- Symmetric across both axes: `abs(p)` folds the input to one quadrant
  before the branch.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d2_sdf_vesica`, `sch tree d2_sdf_vesica`)
- **Consumers:** none yet.
