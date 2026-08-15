# d2_sdf_box

Signed distance from a 2D point to an axis-aligned box of the given
half-extents: negative inside, zero on the boundary, positive outside.
The base rectangle primitive — panels, tiles, UI chrome.

## Visualization

![d2_sdf_box preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale field:
`d2_sdf_box( p, vec2f( 0.28, 0.18 ) )`'s raw signed-distance value is
written straight to each pixel as `vec3f( value )`, clamped to `[0, 1]`,
at `preview_scale = 8`. Solid black fills the rectangle's interior;
brightness grows outward, faster from the long edges than from the
corners since the field is the exact (not approximate) box distance.

This demo is now wired in as a permanent `d2_sdf_box_preview` export,
so the chunk is directly previewable via `sch preview d2_sdf_box` — no
wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `d2_sdf_box` |
| `description` | Signed distance from a 2D point to an axis-aligned box of the given half-extents. |
| `tags` | `category:sdf, dim:2d` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | — (no dependencies; leaf chunk) |
| `export` | `fn d2_sdf_box(p: vec2f, half_extents: vec2f) -> f32`, `fn d2_sdf_box_preview(p: vec2f) -> f32` |

## Nuances

- Exact distance both inside and outside, including at the corners —
  unlike a naive per-axis `abs(p)-b` clamp, the corner region correctly
  measures distance to the nearest vertex, not to a face plane.
- Centered at the origin and axis-aligned: rotate `p` by the inverse of
  the desired box rotation before calling, same convention as every other
  chunk in this family.
- Feeds `d2_sdf_round_box` directly (shrink by `r`, subtract `r`) — see
  that chunk's `depends_on`.

## Relatives

- **Depends on:** none — leaf primitive.
- **Depended on by:** [`d2_sdf_round_box`](../d2_sdf_round_box/readme.md)
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get d2_sdf_box`, `sch tree d2_sdf_box`)
- **Consumers:** none yet.
