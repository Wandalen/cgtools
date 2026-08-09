# Exploration: GPU HAL — Buy vs Build

- **Status**: Open — buy-vs-build not decided; evidence gathering is committed and underway: a slice-by-slice WebGPU `renderer` path is being built, and the HAL surface is extracted from the diff between the WebGL and WebGPU implementations rather than designed up front
- **Opened**: 2026-08-08

## Objective

Determine what should fill the L1 hardware abstraction layer of
[ADR-001](../adr/001_multi_stack_rendering_architecture.md): the one crate
through which every stack engine reaches `minwebgl`, `minwebgpu`, and
`minwgpu` without knowing which backend it runs on — while preserving direct
shader access (canonical source + per-backend override) and one-step
drill-down to the raw driver.

## Approaches Investigated

1. **Adopt `wgpu` as the HAL.** Use `wgpu` for native *and* web (its GLES/
   WebGL2 backend covers browsers without WebGPU). The `min*` web drivers
   become internal details or are retired above L0.
2. **Build an in-house, WebGPU-shaped HAL** over the three `min*` drivers.
   API mirrors WebGPU concepts (device, queue, pipeline, bind group) so the
   WebGPU path is near-zero-cost; the WebGL2 path emulates what it must.
   Canonical shaders in WGSL, transpiled to GLSL ES via `naga` at build time,
   with a per-backend override slot for hand-tuned sources.
3. **Grow `mingl` into the HAL.** Disqualified before comparison: the drivers
   depend on `mingl`, so it sits below them; a HAL must depend on the drivers
   (see ADR-001's alternatives).
4. **No HAL** — keep abstracting per stack at L3 (`Backend`-trait style) and
   per crate. The status quo, kept as the explicit null option.

## Comparison

| Criterion | 1 · `wgpu` | 2 · in-house | 4 · no HAL |
|-----------|------------|--------------|------------|
| Control over WebGL2 output | Low — behind wgpu's translation | Full — `minwebgl` stays first-class | Full |
| wasm binary size | Largest (wgpu + naga at runtime) | Small–medium (naga at build time) | Smallest |
| Native path | Excellent — its home ground | Via `minwgpu` (itself wgpu-based) | None shared |
| Shader access contract | WGSL only; overrides awkward | Canonical WGSL + per-backend override by design | Per crate, ad hoc |
| Maintenance cost | Low (upstream maintained) | High — a whole API surface to own | Zero now, O(backends × features) later |
| Fit with strict layering / drill-down | Poor — wgpu hides the driver | Native fit — designed for it | No L1 layer at all |
| Migration cost for `renderer` | Full rewrite | Substantial rewrite | None |

## Recommendation

Lean **in-house (approach 2)** *if* first-class WebGL2 control and the
shader-access contract stay hard requirements — those are exactly what
`wgpu` costs. Adopt **`wgpu` (approach 1)** instead if native becomes the
primary target or the maintenance budget for an owned HAL is unavailable.
Remain at **no HAL (approach 4)** until a second stack actually needs a
non-WebGL backend — building L1 before then would be speculative.

## Next Steps

- Spike (done, superseded the triangle/quad form): built the d3 renderer's
  opaque path on `minwebgpu` slice by slice (`renderer::webgpu`), extracted
  the `gpu_hal` v0 surface from the webgl-vs-webgpu diff, then implemented a
  WebGL2 backend of that surface and ported the canonical path onto the HAL —
  it now compiles clean against both backends. Measured WebGL2 emulation
  surface: uniform-block/texture-unit introspection by name convention
  (`ub_{g}_{b}`/`tex_{g}_{b}`), per-pass FBO lifecycle, eager state
  application at bind time, GLSL twin shaders (no transpilation yet) —
  moderate, contained in one file-set. Approach 2 (in-house) is working in
  practice; runtime smoke tests per backend remain to run.
- Measure wasm binary size: spike facade vs `wgpu` for the same two scenes.
- Evaluate `naga` build-time WGSL→GLSL ES output against the hand-written
  shaders in `module/helper/renderer/src/webgl/shaders/`.
- On a decision, close this exploration and record it as an ADR.
