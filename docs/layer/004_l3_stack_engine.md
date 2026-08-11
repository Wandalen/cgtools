# Layer: L3 Stack Engine

Where a stack's invariants become running code: one engine per stack,
exposing that stack's rendering vocabulary upward and consuming foundation
layers downward. This is the highest layer that knows about GPU concepts
and the lowest that knows about stack concepts.

### Scope

- **Purpose**: Define the engine layer's role and record the current engines and their seams.
- **Responsibility**: Name each stack's engine, its upward vocabulary, and its downward dependencies (including the accepted pre-HAL violations).
- **In Scope**: `tilemap_renderer` (d2) and `renderer` (d3) as L3 engines.
- **Out of Scope**: The stacks' invariant tables (see [../render_stack/readme.md](../render_stack/readme.md)); scene data consumed from above (see [005_l4_scene_model.md](005_l4_scene_model.md)).

### Role

An engine turns its stack's vocabulary into GPU work (or into declarative
output, where the stack's invariants demand it). The *upward* API is the
stack's own language; the *downward* dependency should be L2/L1 — today it
is L0 directly, the accepted violation tracked in
[../pattern/002](../pattern/002_strict_layering_one_step_drilldown.md).

### Engines Today

| Engine | Stack | Upward vocabulary | Downward seam |
|--------|-------|-------------------|---------------|
| `renderer` | d3 | Scene graph: `Node`/`Scene`/`Mesh`, PBR materials, cameras, lights | Direct `minwebgl` (`renderer::webgl::*` namespace) |
| `tilemap_renderer` | d2 | POD `RenderCommand` stream + assets | `Backend` trait — SVG needs no GPU at all; the WebGL2 adapter uses `minwebgl` |

The two demonstrate the two portability strategies ADR-001 weighs: a trait
seam at the command level (`tilemap_renderer` — backends multiply freely)
versus per-backend namespaces (`renderer` — each backend is a parallel
tree). The architecture keeps the first and dissolves the second onto the
HAL.

### Layers

| File | Relationship |
|------|--------------|
| [003_l2_frame_orchestration.md](003_l2_frame_orchestration.md) | Machinery currently embedded in these engines |
| [005_l4_scene_model.md](005_l4_scene_model.md) | The declarative data engines receive from above |

### Render Stacks

| File | Relationship |
|------|--------------|
| [../render_stack/001_d2.md](../render_stack/001_d2.md) | The invariants `tilemap_renderer` realizes |
| [../render_stack/003_d3.md](../render_stack/003_d3.md) | The invariants `renderer` realizes |

### Sources

| File | Relationship |
|------|--------------|
| `module/helper/renderer/src/webgl/renderer.rs` | d3 engine core |
| `module/helper/tilemap_renderer/src/backend.rs` | d2 engine's `Backend` seam |
