# Doc Definitions

Workspace-scope design documentation — content whose subject spans multiple
crates. Sanctioned by `rulebook.md § Workspace-scope documentation`; anything
scoped to a single crate lives in that crate's own `docs/` instead.

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `adr/` | Accepted ecosystem-level architecture decisions and their alternatives | [adr/readme.md](../adr/readme.md) | 3 |
| `explorations/` | Open multi-crate design investigations that have not yet produced a decision | [explorations/readme.md](../explorations/readme.md) | 1 |
| `layer/` | One identity card per ecosystem layer (L0–L5): contract, occupants, status | [layer/readme.md](../layer/readme.md) | 6 |
| `pattern/` | Reusable cross-crate design rules the ecosystem is built on | [pattern/readme.md](../pattern/readme.md) | 5 |
| `render_stack/` | One identity card per render stack: invariant table, renounced capabilities, members | [render_stack/readme.md](../render_stack/readme.md) | 3 |

## Master Doc Instances Table

| Definition | ID | Name | File |
|-----------|-----|------|------|
| adr | 001 | Multi-Stack Rendering Architecture | [adr/001_multi_stack_rendering_architecture.md](../adr/001_multi_stack_rendering_architecture.md) |
| adr | 002 | In-House GPU HAL | [adr/002_gpu_hal_in_house.md](../adr/002_gpu_hal_in_house.md) |
| adr | 003 | Extend L1 HAL Adoption to the d2 Stack | [adr/003_d2_stack_hal_adoption.md](../adr/003_d2_stack_hal_adoption.md) |
| explorations | 001 | GPU HAL: Buy vs Build | [explorations/001_gpu_hal_buy_vs_build.md](../explorations/001_gpu_hal_buy_vs_build.md) |
| layer | 001 | L0 Drivers | [layer/001_l0_drivers.md](../layer/001_l0_drivers.md) |
| layer | 002 | L1 GPU Hardware Abstraction | [layer/002_l1_gpu_hal.md](../layer/002_l1_gpu_hal.md) |
| layer | 003 | L2 Frame Orchestration | [layer/003_l2_frame_orchestration.md](../layer/003_l2_frame_orchestration.md) |
| layer | 004 | L3 Stack Engine | [layer/004_l3_stack_engine.md](../layer/004_l3_stack_engine.md) |
| layer | 005 | L4 Scene Model | [layer/005_l4_scene_model.md](../layer/005_l4_scene_model.md) |
| layer | 006 | L5 Scene Script and Runners | [layer/006_l5_scene_script_and_runners.md](../layer/006_l5_scene_script_and_runners.md) |
| pattern | 001 | Invariant-Defined Stack | [pattern/001_invariant_defined_stack.md](../pattern/001_invariant_defined_stack.md) |
| pattern | 002 | Strict Layering with One-Step Drill-Down | [pattern/002_strict_layering_one_step_drilldown.md](../pattern/002_strict_layering_one_step_drilldown.md) |
| pattern | 003 | Cross-Stack Bridge via Foundation Resources | [pattern/003_cross_stack_bridge_via_foundation_resources.md](../pattern/003_cross_stack_bridge_via_foundation_resources.md) |
| pattern | 004 | Script-as-Data | [pattern/004_script_as_data.md](../pattern/004_script_as_data.md) |
| pattern | 005 | Script-as-Glue | [pattern/005_script_as_glue.md](../pattern/005_script_as_glue.md) |
| render_stack | 001 | d2 | [render_stack/001_d2.md](../render_stack/001_d2.md) |
| render_stack | 002 | tile | [render_stack/002_tile.md](../render_stack/002_tile.md) |
| render_stack | 003 | d3 | [render_stack/003_d3.md](../render_stack/003_d3.md) |
