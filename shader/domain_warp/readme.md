# domain_warp

Warps a 2D point by two centered [`fbm3`](../fbm3/readme.md) offsets for
organic distortion. Feed the warped point into any noise (or into `fbm3`
again) and straight lattice artifacts dissolve into flowing, marbled,
smoke-like structure — the classic "noise of warped coordinates" trick.

## Visualization

![domain_warp preview](preview.png)

Rendered via the chunk-preview harness's synthesized grayscale
field: `fbm3( domain_warp( p, 0.6 ) ) / 0.875` — plain fbm sampled
through the warp, normalized by fbm3's maximum — is written
straight to `vec3f( value )`, clamped to `[0, 1]`, at
`preview_scale = 8`. Compare with the `fbm3` preview: same field,
but the blobs are smeared into curved filaments. Directly
previewable via `sch preview domain_warp`.

## Parameters

| Field | Value |
|---|---|
| `name` | `domain_warp` |
| `description` | Warps a 2D point by two centered fbm3 offsets for organic distortion. |
| `tags` | `category:noise`, `technique:warp` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | `fbm3` |
| `export` | `fn domain_warp(p: vec2f, strength: f32) -> vec2f`, `fn domain_warp_preview(p: vec2f, strength: f32) -> f32` |

## Nuances

- The two offset channels are the same `fbm3` sampled at `p` and at
  `p + ( 5.2, 1.3 )` — a fixed decorrelation shift (per Quilez's domain
  warping article), cheaper than a dedicated vector-valued fbm.
- `fbm3` outputs `[0, 0.875]`; the `* ( 2.0 / 0.875 ) - 1` remap recenters
  that to `[-1, 1]`, so the warp is unbiased — `strength` never drags the
  whole field diagonally.
- `strength` is in input-space units: the returned point moves at most
  `strength · √2` away from `p`. Values around `0.3–1.0` at unit scale
  read as "organic"; much larger turns to mush.
- Warping is composable: `domain_warp` of `domain_warp` gives the deeply
  folded two-level look at double the (already 3-octave) cost.

## Relatives

- **Depends on:** [`fbm3`](../fbm3/readme.md) (both warp channels), which
  pulls in `value_noise` and `hash21` transitively.
- **Depended on by:** none yet.
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get domain_warp`, `sch tree domain_warp`)
- **Consumers:** none yet.
