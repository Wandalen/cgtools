# ADR-003: Extend L1 HAL Adoption to the d2 Stack (`tilemap_renderer`)

- **Date**: 2026-08-11
- **Status**: Accepted
- **Deciders**: wandalen

## Context

[ADR-001](001_multi_stack_rendering_architecture.md) placed a single hardware
abstraction layer at L1. [ADR-002](002_gpu_hal_in_house.md) closed L1's
buy-vs-build question in-house (`gpu_hal`), but every backend it proved —
WebGPU, WebGL2, native `wgpu` — was evidenced through exactly one consumer,
the d3 engine (`renderer`). [explorations/001](../explorations/001_gpu_hal_buy_vs_build.md)'s
own Recommendation named the trigger for going further explicitly: remain at
no-HAL-for-other-stacks "until a second stack actually needs a non-WebGL
backend — building L1 before then would be speculative."

That trigger has now fired. The d2 stack's engine, `tilemap_renderer`,
already has a `Backend`-trait multi-adapter architecture
([`docs/pattern/001`](../../module/helper/tilemap_renderer/docs/pattern/001_ports_and_adapters_backend_architecture.md))
with SVG, direct-`minwebgl` WebGL2, and terminal-stub adapters — but no path
to WebGPU, native `wgpu`, or a Vulkan-forced native backend at all. A
concrete consumer surfaced the gap directly:
`examples/scene_script/pingpong_animation` (L5, script-as-glue, d2 stack)
wants to swap among WebGL / WebGPU / native `wgpu` / Vulkan-forced /
offscreen / math-only-no-render targets for the same simulated scene. Beyond
the missing backends, no wiring exists at all yet between L5 output and L3
input for this example — `pingpong_animation` emits `Frame` structs to a
Rust callback, never `tilemap_renderer::RenderCommand`s — so today it cannot
render through *any* backend, including the ones (`adapter-svg`,
`adapter-webgl`) that already exist and work for other d2 content.

## Decision

1. **L1 HAL adoption extends to the d2 stack.** `tilemap_renderer` gains two
   new `Backend` adapters mirroring `gpu_hal`'s existing backend set:
   `adapter-webgpu` (browser WebGPU via `gpu_hal`'s `webgpu` feature) and
   `adapter-native` (native `wgpu` via `gpu_hal`'s `native` feature,
   offscreen render + pixel readback — the same proof shape as `renderer`'s
   `triangle_render_readback`). The existing `adapter-svg` /
   `adapter-terminal` / `adapter-webgl` are unchanged; `adapter-webgl` keeps
   its direct `minwebgl` dependency for now (see Alternatives).

2. **A formal no-op adapter.** `tilemap_renderer` gains `adapter-none`: a
   `Backend` implementation that accepts assets/commands and does no GPU or
   document work. This makes "math-only simulation, no rendering" a first-
   class backend selection instead of an ad hoc "just don't call the
   engine" convention repeated at each call site.

3. **Vulkan is a backend-selection detail, not a new adapter.** Forcing
   `wgpu` onto its Vulkan backend happens inside `adapter-native`'s
   construction (an explicit-backend constructor path), mirroring the
   precedent set by `examples/minwgpu/sun_grid_lines_vulkan` (since
   removed) of forcing wgpu's backend bits rather than inventing a
   parallel API surface. This remains true for `tilemap_renderer`'s own
   `adapter-native` — its Vulkan run mode is still `wgpu`-forced, not a new
   adapter, and this decision is unchanged for that consumer.

   > **Scoped update (2026-08-16, [ADR-004](004_native_vulkan_hal_backend.md)):**
   > the orrery family's Vulkan plan named in the original text of this
   > decision — "as a run mode of its native-`wgpu` member" — no longer
   > holds for `examples/orrery/flexible`, which needs a Vulkan option that
   > does not link `wgpu` at all. That consumer gets Vulkan through a new,
   > genuinely `wgpu`-free `gpu_hal` backend (`minvulkan`) instead. This
   > amendment is scoped to that one consumer; `tilemap_renderer`'s
   > `adapter-native` and every other reasoning in this decision are
   > unaffected.

4. **L5→L3 wiring is example-local glue, not a new shared crate.** Compiling
   a script's per-frame output (e.g. `pingpong_animation`'s `Frame`) into
   `RenderCommand`s is written once, directly in the consuming example. No
   general d2 scene-model crate is created speculatively now (see
   Alternatives) — this leaves
   [layer/005](../layer/005_l4_scene_model.md)'s d2 slot exactly as it
   stands today.

5. **Backend selection follows the existing per-adapter feature
   convention.** New adapters are gated the same way as the existing three
   (`adapter-webgpu`, `adapter-native`, `adapter-none` alongside
   `adapter-svg`, `adapter-webgl`, `adapter-terminal`) — the same shape
   `renderer` and `gpu_hal` already use for their own backend features.

## Alternatives Considered

- **Route d2 through `renderer`'s webgpu/native machinery instead of
  extending `tilemap_renderer`.** Rejected: `renderer` is the d3 engine —
  scene graph, PBR materials, a depth buffer — and assumes invariants d2
  explicitly renounces
  ([render_stack/001](../render_stack/001_d2.md)'s Renounced Capabilities).
  Reusing it would cross the invariant-defined-stack boundary
  ([pattern/001](../pattern/001_invariant_defined_stack.md)) rather than
  extend L1 adoption cleanly.
- **Build a general/reusable L4 scene-model crate for d2 now** (a `d2_scene`
  counterpart to `tilemap_scene` / `d3_scene`). Rejected for now — YAGNI:
  exactly one script (`pingpong_animation`) currently needs L5→L3
  compilation. A second concrete consumer needing the same shape is the
  trigger to extract a shared crate, not this one — the same reasoning
  `explorations/001` already used to defer L1 itself until triggered.
- **A dedicated `adapter-vulkan`.** Rejected: Vulkan is a `wgpu` backend
  selection, not a distinct command-stream target;
  `sun_grid_lines_vulkan` (since removed) already established that forcing
  a backend is a constructor-time choice, not a new `Backend` implementation.
- **Migrate `adapter-webgl` onto `gpu_hal` at the same time.** Rejected for
  now: it already works: unwiring it risks regressing a working backend for
  no functional gain the current request needs. Its HAL migration stays an
  open, untriggered question — the same "strangle when triggered" posture
  ADR-002 already accepted for `renderer`'s legacy `webgl` tree.

## Consequences

- `tilemap_renderer` becomes the second L3 engine to depend on `gpu_hal`,
  proving L1's stack-vocabulary-free, WebGPU-shaped contract
  ([layer/002](../layer/002_l1_gpu_hal.md)) against a materially different
  stack shape — d2's flat POD command stream versus d3's scene graph.
- `pingpong_animation` (and any future script wanting a visual or offscreen
  output) gains a small, example-local compilation step from its own
  frame/state shape to `RenderCommand`s. This is glue code, not a new L5
  capability — [layer/006](../layer/006_l5_scene_script_and_runners.md)'s
  contract is unchanged.
- `Backend::Capabilities` must stay honest for the two new adapters, per
  [`docs/pattern/001`](../../module/helper/tilemap_renderer/docs/pattern/001_ports_and_adapters_backend_architecture.md)'s
  existing Consequences for every adapter.
- New adapters' output-correctness invariants (mirroring `renderer`'s
  `opaque_path_renders_lit_quad` / `triangle_render_readback` proofs) get
  their `tilemap_renderer/docs/invariant/` entries when each adapter is
  actually implemented and the guarantee is real — not written speculatively
  ahead of the code.
- `adapter-webgl` stays on direct `minwebgl` until a separate, explicitly
  triggered decision migrates it.

## Related

- [001_multi_stack_rendering_architecture.md](001_multi_stack_rendering_architecture.md) — defines the L1 slot and the layer ladder this extends
- [002_gpu_hal_in_house.md](002_gpu_hal_in_house.md) — the HAL this ADR extends adoption of; its "second stack" trigger condition
- [explorations/001_gpu_hal_buy_vs_build.md](../explorations/001_gpu_hal_buy_vs_build.md) — source of the trigger condition this ADR fires
- [layer/002_l1_gpu_hal.md](../layer/002_l1_gpu_hal.md) — L1's living identity card, updated for this decision
- [layer/004_l3_stack_engine.md](../layer/004_l3_stack_engine.md) — L3's living identity card, updated for this decision
- [render_stack/001_d2.md](../render_stack/001_d2.md) — the d2 stack invariants the new adapters must still honor
- `module/helper/tilemap_renderer/docs/pattern/001_ports_and_adapters_backend_architecture.md` — the adapter architecture the new adapters must follow
- [004_native_vulkan_hal_backend.md](004_native_vulkan_hal_backend.md) — scopes Decision #3's orrery-Vulkan claim down to `tilemap_renderer` only; adds a `wgpu`-free Vulkan path for `examples/orrery/flexible`
