# Doc Definitions

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `algorithm/` | Deterministic computational procedures: animation phase resolution, scene rendering pass | [algorithm/readme.md](../algorithm/readme.md) | 2 |
| `api/` | Public runtime operations a game uses to drive a loaded scene and render it | [api/readme.md](../api/readme.md) | 1 |
| `format/` | RON/serde schema constructs: data model, encoding, version-compatibility contracts | [format/readme.md](../format/readme.md) | 8 |
| `invariant/` | Correctness properties that must always hold, and their enforcement mechanisms | [invariant/readme.md](../invariant/readme.md) | 2 |
| `pitfall/` | Known traps in load-time validation, their failure modes, and mitigations | [pitfall/readme.md](../pitfall/readme.md) | 1 |

## Master Doc Instances Table

| Definition | ID | Name | File |
|--------|-----|------|------|
| algorithm | 001 | Animation Phase & Frame Selection | [algorithm/001_animation_phase_and_frame_selection.md](../algorithm/001_animation_phase_and_frame_selection.md) |
| algorithm | 002 | Scene Rendering Pass | [algorithm/002_scene_rendering_pass.md](../algorithm/002_scene_rendering_pass.md) |
| api | 001 | Renderer Integration API | [api/001_renderer_integration_api.md](../api/001_renderer_integration_api.md) |
| format | 001 | Scene Object Model | [format/001_scene_object_model.md](../format/001_scene_object_model.md) |
| format | 002 | Grid Coordinate System | [format/002_grid_coordinate_system.md](../format/002_grid_coordinate_system.md) |
| format | 003 | Anchor Placement Types | [format/003_anchor_placement_types.md](../format/003_anchor_placement_types.md) |
| format | 004 | Declared Resources | [format/004_declared_resources.md](../format/004_declared_resources.md) |
| format | 005 | Sprite Sources | [format/005_sprite_sources.md](../format/005_sprite_sources.md) |
| format | 006 | Layer Behaviour | [format/006_layer_behaviour.md](../format/006_layer_behaviour.md) |
| format | 007 | Render Pipeline | [format/007_render_pipeline.md](../format/007_render_pipeline.md) |
| format | 008 | Top-Level File Structure | [format/008_top_level_file_structure.md](../format/008_top_level_file_structure.md) |
| invariant | 001 | RenderSpec Referential Integrity | [invariant/001_renderspec_referential_integrity.md](../invariant/001_renderspec_referential_integrity.md) |
| invariant | 002 | Edge and Vertex Canonical Uniqueness | [invariant/002_edge_and_vertex_canonical_uniqueness.md](../invariant/002_edge_and_vertex_canonical_uniqueness.md) |
| pitfall | 001 | Load-Time Validation Is Only Partially Enforced | [pitfall/001_load_time_validation_partially_enforced.md](../pitfall/001_load_time_validation_partially_enforced.md) |
