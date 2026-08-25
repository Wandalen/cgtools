# ADR-002: In-House GPU HAL (`gpu_hal`) Over `wgpu` Adoption

- **Date**: 2026-08-10
- **Status**: Accepted
- **Deciders**: wandalen

## Context

[ADR-001](001_multi_stack_rendering_architecture.md) placed a single hardware
abstraction layer at L1 — the one crate through which every stack engine
reaches the drivers without knowing which backend it runs on — but left *what
fills it* to [explorations/001](../explorations/001_gpu_hal_buy_vs_build.md).
That exploration compared adopting `wgpu` wholesale, building an in-house
WebGPU-shaped crate over the `min*` drivers, growing `mingl` upward
(disqualified on dependency direction), and deferring the layer entirely.

Rather than decide on paper, the exploration committed to evidence: build the
d3 renderer's opaque path slice by slice on WebGPU, extract the HAL surface
from the webgl-vs-webgpu diff, and see what the abstraction actually costs.

## Decision

**Build in-house: `gpu_hal` (`module/helper/gpu_hal`) is the L1 HAL.** The
API mirrors WebGPU concepts (device, queue, pipeline, bind group, pass) so
the WebGPU path is near-zero-cost; the other backends emulate what they must.
`wgpu` is not rejected as a technology — it powers the *native* leg at L0
(via `minwgpu`) underneath the same `gpu_hal` surface; it is rejected only
as the abstraction layer itself.

The decision rests on delivered evidence, not projection — v0 exists with
three working backends behind one surface:

| Backend | Feature | Target | Proven by |
|---------|---------|--------|-----------|
| WebGPU | `webgpu` | wasm32 | `renderer::webgpu` compiles + browser suites |
| WebGL2 | `webgl` | wasm32 | same canonical path, twin GLSL shaders |
| Native `wgpu` | `native` | non-wasm | `triangle_render_readback` pixel test |

`renderer`'s canonical opaque path (PBR + ACES tone mapping) is written once
against this surface and runs on all three; on the native backend it is
pixel-verified end-to-end in the terminal
(`opaque_path_renders_lit_quad`). Verify:

```sh
cargo nextest run -p gpu_hal --features native
cargo nextest run -p renderer --features native
```

## Alternatives Considered

- **Adopt `wgpu` as the HAL.** Rejected for the web: it interposes a
  translation layer over WebGL2 (surrendering the first-class `minwebgl`
  control the `min*` crates exist to provide), costs wasm binary size
  (naga at runtime), accepts WGSL only (the canonical-source +
  per-backend-override shader contract becomes awkward), and hides the
  driver that strict layering's drill-down handles must reach. Adopted
  *inside* the native leg instead, where it is on home ground.
- **Grow `mingl` into the HAL.** Disqualified on dependency direction — the
  drivers depend on `mingl`, so it sits below them; a HAL must sit above
  (ADR-001, alternatives).
- **No HAL** (keep abstracting per stack at L3). Superseded by events: the
  d3 stack already needed WebGPU + WebGL2 from one codebase, and terminal
  pixel-testing needed a third, browserless backend — three backends behind
  one surface is no longer speculative.

## Consequences

- The portability seam ADR-001 promised at L1 is real: `renderer::webgpu` is
  one tree targeting three backends, versus the legacy per-backend
  `renderer::webgl` tree it will strangle.
- Cost accepted: an owned API surface to maintain. Measured emulation burden
  on WebGL2 — uniform-block/texture-unit introspection by name convention
  (`ub_{g}_{b}`/`tex_{g}_{b}`), per-pass FBO lifecycle, eager state
  application, GLSL twin shaders — moderate, contained in one file-set.
- v0 scope is the opaque path only: buffers, 2d textures, samplers, shader
  modules, bind groups, one-color-attachment render passes. Texture upload,
  mipmaps, MSAA, and compute are added when a consumer needs them, not
  before.
- Shaders stay hand-written twins (WGSL + GLSL 300 es). Build-time
  WGSL→GLSL transpilation via `naga` remains an open refinement — it would
  reduce the twin-maintenance cost but is not load-bearing for the decision.
- The wasm-binary-size comparison (spike facade vs `wgpu` for identical
  scenes) was planned but never run; the decision closes on control, shader
  contract, layering fit, and the delivered three-backend v0. If size
  pressure ever appears, measure then.
- Browser-backend runtime pixel tests (the WebGPU/WebGL2 analogues of
  `opaque_path_renders_lit_quad`) remain to run; the browser suites cover
  the canonical path structurally today.

## Related

- [001_multi_stack_rendering_architecture.md](001_multi_stack_rendering_architecture.md) — defines the L1 slot this fills
- [explorations/001_gpu_hal_buy_vs_build.md](../explorations/001_gpu_hal_buy_vs_build.md) — the comparison and spike evidence (closed by this ADR)
- [layer/002_l1_gpu_hal.md](../layer/002_l1_gpu_hal.md) — the layer's living identity card
- `module/helper/gpu_hal/readme.md` — crate-level surface and backend notes
