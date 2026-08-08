# ADR-001: Multi-Stack Rendering Architecture

- **Date**: 2026-08-08
- **Status**: Accepted
- **Deciders**: wandalen

## Context

The workspace aims at a rendering ecosystem that is multilayer (every layer
builds only on the layer directly below it), reaches from a raw
backend driver at the bottom to a scene-as-script representation at the top —
one that is simultaneously parsable and interpretable, renderable both
interactively and off-screen — and targets three backends: WebGL2 and WebGPU
on the web, `wgpu` for native binaries.

Two divergent portability strategies already coexist in the codebase:

- `renderer` duplicates per backend: everything lives under a
  `renderer::webgl::*` namespace and depends directly on `minwebgl`. Porting
  it to WebGPU means a parallel `renderer::webgpu::*` tree — cost grows as
  O(backends × features).
- `tilemap_renderer` abstracts at the command level: a `Backend` trait
  (`load_assets` / `submit` / `output` / `resize` / `capabilities`) consumes a
  POD `RenderCommand` stream, with SVG, WebGL2, and terminal adapters behind
  feature gates. Backends multiply without touching the engine.

Below both sit the drivers (`minwebgl` mature, `minwebgpu` functional,
`minwgpu` embryonic), which all depend on `mingl` as a shared substrate of
backend-independent helpers. There is no hardware abstraction layer: nothing
lets one body of GPU code target all three drivers.

Meanwhile the crates cluster into families with *incompatible* rendering
assumptions — `tilemap_renderer`'s output must stay expressible as SVG, while
`renderer` assumes a depth buffer and HDR light transport. A single universal
engine would have to drop to the weakest common contract or leak union
complexity into every layer.

## Decision

1. **One shared foundation, several stacks.** The ecosystem is organized as a
   shared foundation (drivers + a future hardware abstraction layer) carrying
   multiple *stacks*. A stack is defined by its table of rendering invariants
   and limitations — not by genre or marketing labels
   (see [pattern/001](../pattern/001_invariant_defined_stack.md)).

2. **Strict layering with drill-down.** Each layer depends only on the layer
   directly below; skipping layers is forbidden. Power access is preserved by
   one-step drill-down handles, so direct shader access remains reachable at
   every layer (see [pattern/002](../pattern/002_strict_layering_one_step_drilldown.md)).

   | Layer | Role | Today | Target |
   |-------|------|-------|--------|
   | L5 | Scene script + runners (parsable, interpretable, interactive and off-screen) | `tilemap_scene` (tile stack) | per stack |
   | L4 | Scene model | `tilemap_scene` data model; glTF via `renderer` loaders | per stack |
   | L3 | Stack engine (commands / passes) | `tilemap_renderer` (d2), `renderer` (d3) | per stack |
   | L2 | Frame orchestration (pass scheduling, render targets) | embedded inside L3 crates | shared where invariants allow |
   | L1 | GPU hardware abstraction layer | **missing** | one crate, WebGPU-shaped (see [explorations/001](../explorations/001_gpu_hal_buy_vs_build.md)) |
   | L0 | Drivers | `minwebgl`, `minwebgpu`, `minwgpu` (+ `mingl` substrate) | unchanged |

3. **Three initial stacks — `d2`, `tile`, `d3`.** Rust identifiers cannot
   start with a digit, and `line_tools` already ships `d2`/`d3` modules, so
   dimensional names follow that precedent; extension stacks get domain names.

   | Stack | Defining invariants (summary) | Pinned formally at | Crates today |
   |-------|-------------------------------|--------------------|--------------|
   | `d2` | Planar geometry; Y-up; z-layer draw ordering; vector representability of the command stream; alpha-compositing blend modes | `tilemap_renderer/docs/invariant/` 001, 003, 004 | `tilemap_renderer`, `canvas_renderer`, `line_tools::d2` |
   | `tile` (extends `d2`) | Lattice addresses are primary; compiles to the d2 command set only; deterministic compilation | `tiles_tools/docs/invariant/` 002; `tilemap_scene/docs/invariant/` 003, 004 | `tiles_tools`, `tilemap_scene` |
   | `d3` | 3D transform hierarchy; depth-buffer visibility with order-independent transparency; PBR metallic-roughness baseline; HDR-internal, tone-mapped output | `renderer/docs/invariant/` 001, 002, 003 | `renderer`, `line_tools::d3` |

   Each invariant is *pinned* — with statement, enforcement mechanism, and
   violation consequences — in the `docs/invariant/` collection of the crate
   that enforces it; this ADR only aggregates the tables. New-stack rule: a
   crate whose invariant *contradicts* a stack's table founds a sibling stack;
   one that only *adds* invariants founds an extension stack; anything else is
   an ordinary crate inside an existing stack.

4. **Cross-stack composition only through foundation resources** — textures,
   framebuffers, command/data streams — never through another stack's scene
   abstractions (see [pattern/003](../pattern/003_cross_stack_bridge_via_foundation_resources.md)).

5. **Shader access at every layer.** Each layer exposes the shader surface of
   the layer below through its drill-down handle; the future HAL keeps
   canonical shader source plus a per-backend override slot rather than hiding
   shaders behind fixed pipelines.

## Alternatives Considered

- **One universal engine.** Rejected: d2's vector-representability and d3's
  depth-buffer/HDR invariants contradict each other; a single engine either
  drops to the weakest contract (no HDR, no depth) or leaks a union of both
  into every API.
- **Per-backend duplication as the portability strategy** (generalizing
  `renderer::webgl::*` to `webgpu::*`, `wgpu::*`). Rejected: O(backends ×
  features) cost, drift between trees, and it buries backend differences in
  the highest layers where they are most expensive.
- **`mingl` as the hardware abstraction layer.** Rejected on dependency
  direction: the drivers depend on `mingl`, so it sits *below* them as a
  substrate; a HAL must sit *above* the drivers and depend on them.
- **`wgpu` as the sole backend everywhere.** Rejected as the web-default:
  it inserts a translation layer over WebGL2, costs wasm binary size, and
  removes the direct driver control the `min*` crates exist to provide. It
  remains the intended native path and a serious HAL candidate — kept open in
  [explorations/001](../explorations/001_gpu_hal_buy_vs_build.md).

## Consequences

- Invariant tables give an objective membership test for every existing and
  future crate, and an objective trigger for founding a new stack.
- The portability seam moves down to L1: everything above the HAL is written
  once per stack, not once per backend. `tilemap_renderer`'s `Backend` trait
  already demonstrates the payoff at L3.
- SVG/terminal/off-screen outputs remain guaranteed in the d2 and tile stacks
  because they are invariants, not adapter accidents.
- Cost: the HAL is a substantial build and is *not yet committed* — it stays
  behind [explorations/001](../explorations/001_gpu_hal_buy_vs_build.md) until
  a decision lands. Until then `renderer` stays WebGL-bound; migrating it onto
  the HAL later is a breaking change accepted in advance.
- Strict layering adds hand-off ceremony for power users; drill-down handles
  are the deliberate escape valve.

## Related

- [pattern/001_invariant_defined_stack.md](../pattern/001_invariant_defined_stack.md)
- [pattern/002_strict_layering_one_step_drilldown.md](../pattern/002_strict_layering_one_step_drilldown.md)
- [pattern/003_cross_stack_bridge_via_foundation_resources.md](../pattern/003_cross_stack_bridge_via_foundation_resources.md)
- [explorations/001_gpu_hal_buy_vs_build.md](../explorations/001_gpu_hal_buy_vs_build.md)
- [render_stack/readme.md](../render_stack/readme.md) — one living identity card per stack
- [layer/readme.md](../layer/readme.md) — one living identity card per layer
- Crate-level invariant collections listed in the stack table above
