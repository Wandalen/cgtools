# sdf_op_round

Rounds any signed distance field's sharp corners by radius `r`: the
generic version of the shrink-and-offset trick `d2_sdf_round_box` and
`d3_sdf_round_box` apply to boxes specifically, usable on any shape.

## Visualization

![sdf_op_round preview](preview.png)

Rendered via the chunk-preview harness's synthesized field: the wrapper
applies `sdf_op_round( ·, 0.08 )` to `d2_sdf_box( p, vec2f( 0.22, 0.22
) )`'s value, sampled at a stationary point (this chunk carries
`category:sdf`, so the harness fills the inside, distance-bands the
outside, and holds the sample point still instead of raw-clamping and
drifting it — see
[`shader_chunks_preview_core`](../../module/shader/shader_chunks_preview_core/readme.md)),
at `preview_scale = 8`, with a unit-spaced reference grid (emphasized
axes at the origin) overlaid so scale and center stay legible. Since
`d2_sdf_box` is already an exact field, subtracting `r` from it inflates
the shape outward by `r` everywhere, including at the corners — the
field is visually the same shape as `d2_sdf_round_box` with matching
parameters, confirming this operator is the general mechanism that
chunk builds on.

This demo is now wired in as a permanent `sdf_op_round_preview` export, so the chunk is directly previewable via `sch preview sdf_op_round` — no wrapper file needed.

## Parameters

| Field | Value |
|---|---|
| `name` | `sdf_op_round` |
| `description` | Rounds a signed distance field's corners by shrinking the shape and offsetting outward by r. |
| `tags` | `category:sdf, technique:operator` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | `d2_sdf_box` |
| `export` | `fn sdf_op_round(d: f32, r: f32) -> f32`, `fn sdf_op_round_preview(p: vec2f, box_half_extent: f32, round_radius: f32) -> f32` |

## Nuances

- Only exact when applied to a distance already measured from a *shrunk*
  shape — calling this directly on an unshrunk shape's distance just
  offsets the surface outward by `r` uniformly (still a valid, useful
  operation — a uniform "grow/inflate" — but not the same as rounding a
  corner in place, which needs the shrink baked into the input's own
  parameters, as `d2_sdf_round_box`/`d3_sdf_round_box` do).
- Trivial (`d - r`), but named and chunked separately for discoverability
  and consistent composition with the other `sdf_op_*` combinators.
- Negative `r` is a valid "shrink" in the same sense `sdf_op_onion` uses
  `abs`, though without onion's hollowing.

## Relatives

- **Depends on:** [`d2_sdf_box`](../d2_sdf_box/readme.md).
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get sdf_op_round`, `sch tree sdf_op_round`)
- **Consumers:** none yet.
