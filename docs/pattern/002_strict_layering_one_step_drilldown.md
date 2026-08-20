# Pattern: Strict Layering with One-Step Drill-Down

### Scope

- **Purpose**: Keep the layer graph a chain — every layer depends only on the layer directly below — without sealing power users away from lower-level control.
- **Responsibility**: Define the two coupled rules (one-step dependency; one-step drill-down handle) and the shader-access contract they exist to preserve.
- **In Scope**: Dependency discipline between layers L0–L5 of [ADR-001](../adr/001_multi_stack_rendering_architecture.md) and the escape-hatch design.
- **Out of Scope**: What the layers themselves are (see [../layer/](../layer/readme.md) and ADR-001's layer table); which crates share layers across stacks (see [001_invariant_defined_stack.md](001_invariant_defined_stack.md)).

### Problem

Layered graphics APIs fail in one of two directions. **Leaky**: callers
import any lower layer directly ("just grab the GL context here"), so every
layer couples to every other and no layer can change its substrate. **Sealed**:
the abstraction hides the lower layer entirely, so the first requirement it
cannot express — a custom shader, an exotic pipeline state — forces users to
fork or abandon the engine. Shader access is the recurring casualty: engines
either expose raw handles everywhere or nowhere.

### Solution

Two rules, always adopted together:

1. **One-step dependency.** A crate in layer *n* declares Cargo dependencies
   only on layer *n−1* (plus non-rendering utility crates). Never *n−2*.
   The Cargo dependency graph is the enforcement mechanism — a skip-level
   `use` cannot compile without a skip-level dependency, and that dependency
   is visible in review.
2. **One-step drill-down.** Every wrapper object exposes an explicit handle
   to its layer-(*n−1*) counterpart (the scene exposes its renderer; the
   renderer its HAL device; the HAL device its raw driver context). Reaching
   L0 from L5 is possible — but only as a visible chain of single steps, each
   greppable at the call site.

The flagship application is the **shader-access contract** of ADR-001: shader
source and pipeline state are reachable at every layer, because each layer
hands you the layer below rather than a sealed black box. The future HAL
carries canonical WGSL plus a per-backend override slot for the same reason.

### Applicability

Apply to every rendering-stack crate relationship in the workspace. Utility
crates (`ndarray_cg`, `browser_input`, `error_tools`, …) are exempt — they are
substrate, not layers. `mingl` is likewise substrate *below* the drivers, not
a layer above them.

### Consequences

- Substrate swaps stay local: only layer *n+1* sees layer *n*'s API.
- Abstraction leaks are auditable — grep for drill-down accessor calls to
  find every place an app bypasses a layer, and how deep.
- Power users never hit a wall; they hit a visible, reviewable staircase.
- Cost: handle-plumbing boilerplate on every wrapper type, and occasional
  two-step ceremony where a leaky design would be one line.
- Current known violations to burn down: `renderer`'s legacy `webgl` tree,
  `tilemap_renderer`'s WebGL2 adapter, and `line_tools` all depend directly
  on an L0 driver rather than routing through L1. L1 v0 now exists
  (`gpu_hal`) and both `renderer`'s canonical opaque path and
  `tilemap_renderer`'s `adapter-webgpu`/`adapter-native` already route
  through it (see [../layer/002_l1_gpu_hal.md](../layer/002_l1_gpu_hal.md));
  these three remaining direct-to-L0 paths are accepted until strangled
  onto the HAL.

### ADRs

| File | Relationship |
|------|--------------|
| [../adr/001_multi_stack_rendering_architecture.md](../adr/001_multi_stack_rendering_architecture.md) | Adopts this pattern; its layer table defines the *n* levels this pattern chains |

### Features

| File | Relationship |
|------|--------------|
| [../../module/helper/gpu_hal/docs/feature/003_shader_modules_and_render_pipelines.md](../../module/helper/gpu_hal/docs/feature/003_shader_modules_and_render_pipelines.md) | Named `ShaderSource` before this pattern doc existed — canonical WGSL plus a per-backend override slot is the shader-access contract's flagship instance |

### Patterns

| File | Relationship |
|------|--------------|
| [../../module/helper/gpu_hal/docs/pattern/001_enum_per_backend_dispatch_one_step_drilldown.md](../../module/helper/gpu_hal/docs/pattern/001_enum_per_backend_dispatch_one_step_drilldown.md) | The resource-handle-level instantiation of this pattern within the HAL crate |

### Render Stacks

| File | Relationship |
|------|--------------|
| [../render_stack/002_tile.md](../render_stack/002_tile.md) | The conforming one-step dependency chain (`tilemap_scene` → `tilemap_renderer` only) |

### Sources

| File | Relationship |
|------|--------------|
| `module/helper/tilemap_scene/Cargo.toml` | Existing conforming chain: scene (L4/L5) depends on `tilemap_renderer` (L3) only — no driver dependencies |
| `module/helper/renderer/Cargo.toml` | First accepted violation: legacy `webgl` tree depending directly on the L0 driver `minwebgl` |
| `module/helper/tilemap_renderer/Cargo.toml` | Second accepted violation: WebGL2 adapter depending directly on the L0 driver `minwebgl` |
| `module/helper/line_tools/Cargo.toml` | Third accepted violation: optional dependency on the L0 driver `minwebgl` |

### Tests

No dedicated test enforces the dependency chain today; the Cargo graph plus
review are the enforcement. A workspace lint (dependency-allowlist per layer)
is a candidate hardening step once the HAL exists.
