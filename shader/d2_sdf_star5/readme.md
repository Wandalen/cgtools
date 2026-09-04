# d2_sdf_star5

Signed distance from a 2D point to a 5-pointed star of outer radius `r`
and inner-radius factor `rf` (inner vertex radius as a fraction of `r`).
Two point-reflection folds around the star's two symmetry axes reduce the
ten-edge outline to one edge test.

## Visualization

![d2_sdf_star5 preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field:
`d2_sdf_star5( p, 0.3, 0.5 )`'s raw signed-distance value is written
straight to each pixel as `vec3f( value )`, clamped to `[0, 1]`, at
`preview_scale = 8`. Solid black fills the star's interior (apex up);
brightness grows outward from its ten alternating outer/inner edges.

This demo is now wired in as a permanent `d2_sdf_star5_preview` export, so the chunk is directly previewable via `sch preview d2_sdf_star5` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d2_sdf_star5` |
| `description` | Signed distance from a 2D point to a 5-pointed star of outer radius r and inner-radius factor rf. |
| `tags` | `category:sdf, dim:2d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d2_sdf_star5(p: vec2f, r: f32, rf: f32) -> f32`, `fn d2_sdf_star5_preview(p: vec2f, outer_radius: f32, inner_radius_factor: f32) -> f32` |

## Nuances

- `rf` around `0.5` gives a classic 5-point star; near `1.0` approaches a
  decagon, near `0.0` approaches a thin pinwheel — clamp to a sane range
  (roughly `[0.3, 0.8]`) for a recognizable star.
- Fixed to 5 points and apex-up orientation — an N-point generalization is
  Tier B (deferred; needs a per-point angular fold instead of two fixed
  constants).
- The `k1`/`k2` constants are `cos`/`sin` of `72°` and its mirror — precomputed
  for the specific 5-fold symmetry, not derived from a general N at runtime.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d2_sdf_star5`, `sch tree d2_sdf_star5`)
- **Consumers:** none yet.
