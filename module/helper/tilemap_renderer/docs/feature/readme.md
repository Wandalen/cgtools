# Feature Doc Definition

A **feature** instance documents one cohesive slice of the crate's public API. In `tilemap_renderer`, each feature is one of the backend adapters — SVG, WebGL2, terminal, none, WebGPU, or native — that let a single command stream render to a different target, and its instance acts as a navigational hub over that adapter's source, invariants, patterns, and known pitfalls. This collection holds one instance per feature; the table below is the index into them.

### Scope

- **Purpose**: `tilemap_renderer`'s backend adapters exist to let one command stream render to SVG, WebGL2, a terminal, no target at all (math-only simulation), or — via the L1 HAL — WebGPU or native `wgpu`.
- **Responsibility**: Document each backend adapter as a navigational hub over its source, invariants, patterns, and known pitfalls.
- **In Scope**: The six feature-gated `Backend` implementations shipped (or stubbed) in this crate.
- **Out of Scope**: The shared core command/asset vocabulary, which is not adapter-specific (see `src/types.rs`, `src/commands.rs`, `src/assets.rs` directly, and [pattern/001](../pattern/001_ports_and_adapters_backend_architecture.md) for the trait boundary).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [SVG Backend Adapter](001_svg_backend_adapter.md) | Generates SVG 1.1 documents from a command stream | ⚠️ |
| 002 | [WebGL2 Backend Adapter](002_webgl2_backend_adapter.md) | Hardware-accelerated sprite/mesh/batch rendering on `wasm32` | ⚠️ |
| 003 | [Terminal Backend Adapter](003_terminal_backend_adapter.md) | ASCII/Unicode terminal rendering | ⏸️ |
| 004 | [None Backend Adapter](004_none_backend_adapter.md) | Complete no-op — math-only simulation, no rendering | ✅ |
| 005 | [WebGPU Backend Adapter](005_webgpu_backend_adapter.md) | Sprite rendering in-browser via `gpu_hal`'s WebGPU surface | ⚠️ |
| 006 | [Native Backend Adapter](006_native_backend_adapter.md) | Offscreen sprite rendering via `gpu_hal`'s native `wgpu` surface, pixel-verified | ✅ |
