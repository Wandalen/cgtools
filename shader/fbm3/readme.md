# fbm3

3-octave fractal Brownian motion built on `value_noise`, with tunable
per-octave frequency growth (`lacunarity`) and amplitude decay (`gain`), in
`[0, 0.5*(1+gain+gain²)]`.

## Visualization

![fbm3 preview](preview.png)

256×256, evaluated over the same continuous domain as
[`value_noise`](../value_noise/readme.md) (`[0, 8) × [0, 8)`, 32 px per unit
cell at octave 1), mapped to grayscale. The preview harness writes this
chunk's raw output straight to `vec3f( value )`, clamped to `[0, 1]` — no
per-chunk rescaling (see `shader_chunks_preview_core`'s generic
`harness_synthesize`). At the default `gain = 0.5` the true output ceiling
is `0.875`, safely under `1.0`, so the image never clips; drag `gain` toward
`1.0` and the ceiling rises to `1.5`, so bright regions visibly clip to
white; drag it toward `0.0` and the ceiling falls to `0.5`, so the image
never reaches full white. `lacunarity` changes how quickly finer octaves
appear relative to the base one, independent of `gain`. Compare against
`value_noise`'s preview: `fbm3` layers three progressively finer copies of
the same noise, giving visibly richer, more fractal-looking detail than a
single `value_noise` octave.

## Parameters

| Field | Value |
|---|---|
| `name` | `fbm3` |
| `description` | 3-octave fractal Brownian motion built on `value_noise`, with tunable per-octave frequency growth (`lacunarity`) and amplitude decay (`gain`), in `[0, 0.5*(1+gain+gain²)]`. |
| `tags` | `category:noise`, `technique:fractal` |
| `stage` | — (plain callable function, not an entry point) |
| `depends_on` | `value_noise` |
| `export` | `fn fbm3(p: vec2f, lacunarity: f32, gain: f32) -> f32` |

## Nuances

- Standard fractal-Brownian-motion construction: 3 octaves, each one
  scaling frequency by `lacunarity` (`//@ param:`, range `[1, 3]`, default
  `2.0` — this range's midpoint) while scaling amplitude by `gain` (`//@
  param:`, range `[0, 1]`, default `0.5` — this range's midpoint) relative
  to the previous octave. Both defaults reproduce this chunk's original
  fixed behavior (frequency doubling, amplitude halving: `0.5`, `0.25`,
  `0.125`) exactly.
- Fixed at exactly 3 octaves — not parameterized by an octave count. The
  name encodes this directly (`fbm3`, not a generic `fbm(p, octaves)`); a
  4th octave would need a new chunk (e.g. `fbm4`), not a parameter to this
  one.
- The three amplitudes sum to `0.5*(1 + gain + gain²)` — `0.875` at the
  default `gain = 0.5`, this function's original theoretical maximum before
  `gain` existed. Now genuinely dynamic: `0.5` at `gain = 0` (only the first
  octave contributes) up to `1.5` at `gain = 1` (all three octaves contribute
  equally). Documented in the `//@ description:` line as a formula, not a
  fixed number, since it depends on a value only known at draw time.
- Reuses [`value_noise`](../value_noise/readme.md) (and transitively
  [`hash21`](../hash21/readme.md)) for every octave — no separate hashing
  primitive is introduced at this layer.

## Relatives

- **Depends on:** [`value_noise`](../value_noise/readme.md) (called three
  times, at increasing frequency and decreasing amplitude).
- **Depended on by:** [`domain_warp`](../domain_warp/readme.md) (samples
  it twice for the warp offsets); also consumed directly by downstream
  fragment shaders (e.g. `examples/orrery/webgpu`'s
  `shader/scene_fragment.wgsl`).
- **Collection index:** [shader/](../readme.md)
- **Bundled by:** [`shader_chunks_core`](../../module/shader/shader_chunks_core/readme.md)
- **Inspect/compose via CLI:** [`shader_chunks`](../../module/shader/shader_chunks/readme.md)
  (`sch get fbm3`, `sch tree fbm3`)
- **Consumer:** [`examples/orrery/webgpu`](../../examples/orrery/webgpu/readme.md)
