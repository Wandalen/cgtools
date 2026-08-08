# Format Doc Definition

### Scope

- **Purpose**: Navigational hub for `tilemap_scene`'s RON/serde data-format specification.
- **Responsibility**: Document each schema construct's data model, encoding, and version-compatibility contract.
- **In Scope**: Object/layer model, grid coordinates, anchors, declared resources, sprite sources, layer behaviour, render pipeline, top-level file structure.
- **Out of Scope**: Runtime algorithms operating on this data (see `algorithm/`), the programmatic API surface (see `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Scene Object Model](001_scene_object_model.md) | `Object`/`ObjectLayer` schema and id/state namespace rules | ✅ |
| 002 | [Grid Coordinate System](002_grid_coordinate_system.md) | Tiling strategy, axial coordinates, pixel conversion | ⚠️ |
| 003 | [Anchor Placement Types](003_anchor_placement_types.md) | The 6 `Anchor` variants and their source-compatibility rules | ✅ |
| 004 | [Declared Resources](004_declared_resources.md) | `Asset`/`Tint`/`Animation`/`Effect` | ✅ |
| 005 | [Sprite Sources](005_sprite_sources.md) | The 9 `SpriteSource` variants and rule-resolution order | ✅ |
| 006 | [Layer Behaviour](006_layer_behaviour.md) | Tint/blend/effects/alpha/parallax | ⚠️ |
| 007 | [Render Pipeline](007_render_pipeline.md) | `RenderPipeline`/`PipelineLayer`/`SortMode` | ⚠️ |
| 008 | [Top-Level File Structure](008_top_level_file_structure.md) | `RenderSpec`/`SceneSnapshot` and versioning | ⚠️ |

Status ⚠️ marks a doc instance that discloses at least one currently-unenforced normative rule (see each file's Version Compatibility section, and `pitfall/001`).
