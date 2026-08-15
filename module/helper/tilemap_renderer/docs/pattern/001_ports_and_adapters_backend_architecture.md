# Pattern: Ports and Adapters Backend Architecture

### Scope

- **Purpose**: Decouple rendering-command authoring from any specific graphics technology so one command stream can drive multiple output backends.
- **Responsibility**: Document the hexagonal (Ports and Adapters) shape of the crate — the core/adapter split, the `Backend` trait boundary, and the trade-offs it implies.
- **In Scope**: The core-vs-adapter split, the `Backend` trait contract, and why the crate is one feature-gated crate rather than separate adapter crates.
- **Out of Scope**: Per-backend capability status and implementation detail (see the Features reference section below).

### Problem

Applications that render 2D content are commonly locked into a single rendering backend (a specific GPU API, a specific document format, a specific terminal library). Porting to a new target, or supporting several targets from one codebase, means re-authoring the drawing logic per backend. `tilemap_renderer` exists to let a caller define a rendering command stream once and have it processed by whichever backend is enabled, without the core command/asset vocabulary depending on any one graphics technology.

### Solution

The crate is structured as **Ports and Adapters** (hexagonal architecture):

- **Core** (`types`, `commands`, `assets`, `backend`) is platform-independent and carries zero graphics dependencies. It defines the "port": a flat, ordered command stream (`&[RenderCommand]`) of POD (`Copy + Clone + Debug`) command structs — path, text, mesh, sprite, batch-lifecycle, and group commands — plus an `Assets` container (images, sprites, geometries, gradients, patterns, clip masks, paths) loaded once before rendering.
- The **`Backend` trait** is the single seam between core and adapters. Conceptually it exposes five operations: load assets into backend-native resources, submit a command slice for processing, retrieve the rendered output, resize the target, and report a `Capabilities` value so callers can discover at runtime which command/asset features a given backend actually honors (rather than only discovering gaps by trial and error). One trait with a single `submit(&[RenderCommand])` entry point was chosen over a per-command-type dispatch interface (e.g. separate `draw_path`/`draw_text`/... methods) to keep the seam small and let the command stream itself, not the trait, carry the vocabulary.
- **Adapters** (SVG, WebGL2, Terminal, None, WebGPU, Native) each implement `Backend` for one rendering technology — or, for None, no technology at all — and are feature-gated (`adapter-svg`, `adapter-webgl`, `adapter-terminal`, `adapter-none`, `adapter-webgpu`, `adapter-native`) rather than published as separate crates — see the Features reference section below for each adapter's actual implementation status. The latter two route through the L1 HAL (`gpu_hal`) rather than a driver crate directly ([../../../../../docs/adr/003_d2_stack_hal_adoption.md](../../../../../docs/adr/003_d2_stack_hal_adoption.md)).

### Applicability

Fits when a caller wants to author one rendering-command stream and target more than one output technology from it — e.g. producing both a static SVG export and a live WebGL2 render of the same scene, or adding a terminal preview without touching call sites that already emit commands. It is a worse fit for a caller that only ever needs one backend and wants to use that backend's native idioms directly, since the flat command stream is deliberately generic and doesn't expose backend-specific features that don't fit the shared vocabulary (e.g. WebGL-specific shader effects have no command representation).

### Consequences

- A new capability (e.g. a new command variant) must be meaningful across backends in principle, or it becomes a command that most adapters silently or explicitly no-op — see the Features reference section for how the WebGL2 adapter currently handles command variants it doesn't yet implement.
- `Capabilities` lets a caller detect a backend's real support at runtime instead of assuming full coverage from the trait's existence alone; every adapter must keep its `capabilities()` return value honest as its implementation evolves.
- Feature-gating adapters inside one crate (rather than splitting them into separate crates) simplifies dependency management for callers that only need one backend, at the cost of all adapter source living in the same crate regardless of which features a given build enables.
- Because core has zero graphics dependencies, adding or changing an adapter never affects a caller that only depends on the core command/asset types.

### Features

| File | Relationship |
|------|--------------|
| [feature/001_svg_backend_adapter.md](../feature/001_svg_backend_adapter.md) | Implements `Backend` for SVG 1.1 document generation |
| [feature/002_webgl2_backend_adapter.md](../feature/002_webgl2_backend_adapter.md) | Implements `Backend` for hardware-accelerated WebGL2 rendering (partial) |
| [feature/003_terminal_backend_adapter.md](../feature/003_terminal_backend_adapter.md) | Implements `Backend` for terminal output (stub only) |
| [feature/004_none_backend_adapter.md](../feature/004_none_backend_adapter.md) | Implements `Backend` as a complete no-op (math-only simulation) |
| [feature/005_webgpu_backend_adapter.md](../feature/005_webgpu_backend_adapter.md) | Implements `Backend` for WebGPU rendering via `gpu_hal` (partial — no real pixel upload yet) |
| [feature/006_native_backend_adapter.md](../feature/006_native_backend_adapter.md) | Implements `Backend` for offscreen native `wgpu` rendering via `gpu_hal`, pixel-verified |

### Sources

| File | Relationship |
|------|--------------|
| `src/backend.rs` | `Backend` trait, `Output`, `Capabilities`, `RenderError` |
| `src/commands.rs` | `RenderCommand` and the POD command vocabulary shared by all adapters |
| `src/assets.rs` | `Assets` container and asset types loaded once before rendering |
| `src/types.rs` | `Transform`, `ResourceId<T>`, `RenderConfig`, and shared enums |
| `src/adapters/mod.rs` | Feature-gated adapter re-exports |

### Tests

| File | Relationship |
|------|--------------|
| `tests/backend_test.rs` | `Backend` trait contract (`assets_load`/`submit`/`output`/`resize`/`Capabilities::default`), all `RenderError` variants, exercised against local test-double backends |
