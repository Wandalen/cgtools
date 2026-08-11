# Layer Doc Definition

The rendering ecosystem is built as a ladder of **layers**, L0 through L5, each one a distinct responsibility that depends only on the layer directly beneath it — the full ladder is introduced in [ADR-001](../adr/001_multi_stack_rendering_architecture.md) and summarized in the [workspace rulebook](../../rulebook.md#rendering-layer-placement). This collection holds one **instance** — one identity card — per layer: a dedicated file recording that layer's contract, which crates currently occupy it, and what may depend on it. The table below is the index into those six cards.

### Scope

- **Purpose**: One instance per layer of the rendering architecture (L0–L5) — the layer's role, contract, current occupants, and what may depend on it.
- **Responsibility**: Keep each layer's living definition in one place, recording whether it is ✅ operating today, 🔄 partially built or embedded in another layer, or ⏸️ a reserved slot with no crate yet (legend repeated below the table).
- **In Scope**: The six layers adopted by [ADR-001](../adr/001_multi_stack_rendering_architecture.md).
- **Out of Scope**: The dependency and drill-down rules *between* layers (see [../pattern/002_strict_layering_one_step_drilldown.md](../pattern/002_strict_layering_one_step_drilldown.md)); the vertical stack membership of crates (see [../render_stack/readme.md](../render_stack/readme.md)).

### Overview Table

| ID | Name | Purpose | Crates | Status |
|----|------|---------|--------|--------|
| 001 | [L0 Drivers](001_l0_drivers.md) | Thin, backend-faithful wrappers over raw GPU APIs | `minwebgl`, `minwebgpu`, `minwgpu` (+ `mingl` substrate below) | ✅ |
| 002 | [L1 GPU Hardware Abstraction](002_l1_gpu_hal.md) | One API over all drivers — v0 backs `renderer`'s canonical path, WebGPU + WebGL2 ( compile-verified ) | `gpu_hal` | 🔄 v0 |
| 003 | [L2 Frame Orchestration](003_l2_frame_orchestration.md) | Pass scheduling and render-target management | `frame_graph` (reserved); logic embedded in `renderer`, `tilemap_renderer` | 🔄 embedded |
| 004 | [L3 Stack Engine](004_l3_stack_engine.md) | Per-stack engines turning stack vocabulary into GPU work | `tilemap_renderer` (d2), `renderer` (d3) | ✅ |
| 005 | [L4 Scene Model](005_l4_scene_model.md) | Declarative, serializable scene data | `tilemap_scene` (tile); glTF via `renderer` loaders (d3); `d3_scene` (reserved) | 🔄 partial |
| 006 | [L5 Scene Script and Runners](006_l5_scene_script_and_runners.md) | Parsable-and-interpretable scenes, interactive or off-screen | `tilemap_scene`, `scene_script`; `d3_scene` (reserved) | 🔄 partial |

Status legend: ✅ operating today · 🔄 exists partially / embedded in another layer · ⏸️ reserved slot (blank crate), not yet built.
