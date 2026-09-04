# Layer: L4 Scene Model

Declarative scene data: serializable, validate-able descriptions of *what
exists* — with no rendering code and no GPU dependency. An L4 model can be
loaded, inspected, diffed, and validated entirely off-GPU; only handing it
to an L3/L5 consumer produces pixels.

### Scope

- **Purpose**: Define the scene-model layer's role and record which stacks have one today.
- **Responsibility**: Name the model formats per stack and their no-rendering-code boundary.
- **In Scope**: `tilemap_scene`'s RON model (tile stack); glTF as the de facto d3 model.
- **Out of Scope**: Executing/compiling the model into frames (see [006_l5_scene_script_and_runners.md](006_l5_scene_script_and_runners.md)); the engines consuming the result (see [004_l3_stack_engine.md](004_l3_stack_engine.md)).

### Role

- **Declarative**: describes entities, not draw calls.
- **Serializable**: a file format (RON, glTF) is the canonical form, not an
  in-memory object graph.
- **Validate-able without a GPU**: model loading and validation must work
  headless — this is what makes L4 testable and toolable. This is the
  invariant `tilemap_scene` meets; the d3 occupant below does not (its
  loader requires a live GPU context just to parse).

### Occupants per Stack

| Stack | Model | State |
|-------|-------|-------|
| tile | `tilemap_scene`'s RON scene model (`RenderSpec`/`SceneSnapshot` — layers, palettes, variants; not the in-memory `Scene` runtime graph, which has no `Serialize`/`Deserialize` derive, see Sources below) | ✅ Dedicated crate; GPU-free by dependency surface — `tiles_tools`' default-on `animation` feature, which transitively pulled in `minwebgl` while going unused, was removed (task 117) ([`tilemap_scene` invariant/003](../../module/helper/tilemap_scene/docs/invariant/003_compiles_to_renderer_commands_only.md)). `validate()` now enforces the majority of its rule set rather than being a no-op: all 11 `RenderSpec` rules (id uniqueness, ref resolution for pipeline-layer/asset/animation/tint/effect/`connects_with`, `NeighborBitmask`/nesting/tiling-whitelist legality, default-state existence, reserved-id) and 2 of 5 `SceneSnapshot` rules (tile-source exclusivity, entity-owner bounds) are enforced by `validate()` itself and tested (`tests/scene_model_test.rs`); the remaining 3 (palette→object-id coverage, per-instance object-id resolution, `initial_global_tint` resolution) need both `SceneSnapshot` and `RenderSpec` together, which `Validate::validate(&self)` can't take — they're enforced instead by `Scene::from_snapshot` (`src/scene.rs`) via `SnapshotLoadError::UnknownObject`/`UnknownTint`, tested in `tests/scene_model_compile_test.rs` (see [`tilemap_scene` invariant/001](../../module/helper/tilemap_scene/docs/invariant/001_renderspec_referential_integrity.md)) |
| d2 (general) | None dedicated — content arrives as direct `tilemap_renderer` commands or via `scene_script` | 🔄 Gap accepted; no committed need yet |
| d3 | glTF, consumed through `renderer`'s file-based loaders, or assembled procedurally by `primitive_generation`'s `primitives_data_to_gltf` — same `GLTF` struct, a second producer of the same artifact type ([task/decisions.md Q-04](../../task/decisions.md#q-04--primitive_generations-l0-l5-ladder-placement)) | 🔄 De facto: the format is standard, but there is no cgtools-owned model crate wrapping it; both producers require a live `WebGl2RenderingContext` — `renderer`'s `load()` to parse, `primitive_generation`'s `primitives_data_to_gltf` to build GL buffers directly — so neither is off-GPU-validatable end-to-end, though pure sub-surfaces are: light-extraction (`light_list_get`) is now natively tested off-GPU (task 118), and URI resolution (`asset_uri_resolve`) has its own pre-existing native coverage in `gltf_loader_tests.rs` (6 cases: relative-path joining, `blob:`/`data:`/`https://` URIs, absolute paths, empty-folder-path edge case); animation-channel decoding (`channel_decode`/`vec3_sequence`, in the animation-specific loader below) is now natively tested too (`gltf_animation_loader_test.rs`, task 223; extended by task 299 to also cover `weights_sequence`/`quat_sequence`); vertex-attribute descriptor computation (`attribute_descriptor_make`) is natively tested (`gltf_attribute_descriptor_test.rs`, task 299); node/skeleton/scene assembly (`nodes_create`/`skeletons_attach`/`scenes_create`) is natively tested (`gltf_node_scene_test.rs`, task 299); skeleton transform resolution (`skeleton_transforms_data_load`) is natively tested (`skeleton_tests.rs`), as is its sibling morph-target displacement packer (`skeleton_displacements_data_load`, `gltf_skeleton_displacements_test.rs`, task 299), and material-variant caching (`material_variation_resolve`, `gltf_material_variation_test.rs`) — per-node light lookup (`light_get`, distinct from the document-level `light_list_get` above) is now natively tested too (`gltf_light_parsing_test.rs`, regression coverage for BUG-189/BUG-172) — `primitive_generation`'s pure geometry/text-generation sub-surface is natively tested too, entirely apart from `primitives_data_to_gltf`'s GL-bound path: curve-to-geometry conversion (`curve_to_geometry`), plane-mesh generation (`plane_to_geometry`), font-contour fill-geometry extraction (`contours_to_fill_geometry`, `font-processing` feature), and vector-path flattening (`path_to_points`, `text` feature) in `primitive.rs` — all four exported zero-GL via `mod_interface!` — plus text-mesh layout (`text_to_mesh`) and text-contour-mesh layout (`text_to_countour_mesh`) in `text/ufo.rs` (both `font-processing`-feature-gated, like `contours_to_fill_geometry` above), both returning `Vec<PrimitiveData>` directly and likewise exported zero-GL via `mod_interface!`; and box/cylinder/torus/icosphere raw mesh generation (`box_mesh`/`cylinder_mesh`/`torus_mesh`/`icosphere`) in `solid.rs`, also zero-GL and exported via `mod_interface!`; covered by 22 native `#[test]` functions (no GPU context needed) across 8 files: `curve_to_geometry_test.rs` (3 cases, task 018 regression), `contours_to_fill_geometry_test.rs` (2 cases, task 018 regression), `path_to_points_test.rs` (2 cases, BUG-127 regression), `geometry_normal_attribute_test.rs` (3 cases, BUG-217 regression), `font_bounding_box_union_test.rs` (1 case, BUG-216 regression), `ufo_glif_point_type_test.rs` (2 cases, BUG-128/BUG-215 regression), `ufo_text_advance_test.rs` (1 case, BUG-129 regression), and `solid_test.rs` (8 cases) |

`d3_scene` (`module/blank/d3_scene/`) reserves the slot for a d3-owned
scene model + script, gated on a committed scene-file requirement.

### Layers

| File | Relationship |
|------|--------------|
| [004_l3_stack_engine.md](004_l3_stack_engine.md) | The layer that renders what L4 describes |
| [006_l5_scene_script_and_runners.md](006_l5_scene_script_and_runners.md) | The layer that compiles/executes L4 models over time |

### Sources

| File | Relationship |
|------|--------------|
| `module/blank/d3_scene/` | Reserved d3 scene-layer slot |
| `module/helper/renderer/src/webgl/loaders/gltf.rs` | glTF ingestion — the de facto d3 model boundary; now enforces `extensionsRequired` (`required_extensions_check`, run right after parse) against the extensions the loader actually implements (`KHR_lights_punctual`, `KHR_materials_specular`), refusing to silently produce incomplete output for assets requiring anything else — tested by `tests/gltf_extensions_required_test.rs` |
| `module/helper/renderer/src/webgl/animation/loaders/gltf.rs` | Animation-specific glTF ingestion, alongside the main loader above |
| `module/helper/renderer/tests/gltf_loader_tests.rs` | Native, off-GPU coverage for `asset_uri_resolve`'s pure URI-resolution sub-surface |
| `module/helper/renderer/tests/gltf_light_parsing_test.rs` | Native, off-GPU coverage for `light_list_get`'s pure light-extraction sub-surface and `light_get`'s per-node lookup sub-surface (the latter's 2 tests are BUG-189/BUG-172 regression coverage) |
| `module/helper/renderer/tests/gltf_animation_loader_test.rs` | Native, off-GPU coverage for the animation loader's `channel_decode`/`vec3_sequence` pure sub-surfaces (task 223), extended by task 299 to also cover `weights_sequence`/`quat_sequence` |
| `module/helper/renderer/tests/gltf_attribute_descriptor_test.rs` | Native, off-GPU coverage for `attribute_descriptor_make`'s pure vertex-attribute-descriptor sub-surface (task 299) |
| `module/helper/renderer/tests/gltf_node_scene_test.rs` | Native, off-GPU coverage for `nodes_create`/`skeletons_attach`/`scenes_create`'s pure node/skeleton/scene-assembly sub-surfaces (task 299) |
| `module/helper/renderer/tests/gltf_skeleton_displacements_test.rs` | Native, off-GPU coverage for `skeleton_displacements_data_load`'s pure morph-target displacement-packing sub-surface (task 299) |
| `module/helper/renderer/tests/skeleton_tests.rs` | Native, off-GPU (`pure_tests` module) coverage for `skeleton_transforms_data_load`'s pure transform-resolution sub-surface; also carries wasm/browser-context skeleton-loading tests |
| `module/helper/renderer/tests/gltf_material_variation_test.rs` | Native, off-GPU coverage for `material_variation_resolve`'s pure material-variant-caching sub-surface (BUG-245 regression) |
| `module/helper/primitive_generation/src/primitive.rs` | Pure curve/plane/contour geometry generation (`curve_to_geometry`, `contours_to_fill_geometry`, `plane_to_geometry`, `path_to_points`) feeding `primitive_data.rs` below — zero-GL, natively tested |
| `module/helper/primitive_generation/src/text/ufo.rs` | Pure text-mesh generation (`text_to_mesh`, `text_to_countour_mesh`) feeding `primitive_data.rs` below — zero-GL, natively tested |
| `module/helper/primitive_generation/src/solid.rs` | Pure raw-mesh generation (`box_mesh`, `cylinder_mesh`, `torus_mesh`, `icosphere`) — zero-GL, natively tested (`solid_test.rs`, 8 cases) |
| `module/helper/primitive_generation/src/primitive_data.rs` | `primitives_data_to_gltf` — the second, procedural glTF producer ([Q-04](../../task/decisions.md#q-04--primitive_generations-l0-l5-ladder-placement)) |
| `module/helper/tilemap_scene/src/spec.rs` + `src/snapshot.rs` | The tile stack's declarative model ( `RenderSpec` / `SceneSnapshot`, RON-deserializable ) — not `scene.rs`, which is the runtime/retained-mode counterpart with no `Serialize`/`Deserialize` derive |
| `module/helper/tilemap_scene/tests/scene_model_test.rs` | Native off-GPU coverage: spec/scene parsing and validation (unknown pipeline layer, unknown asset references, missing default state, reserved IDs) |
| `module/helper/tilemap_scene/tests/ron_syntax_error_test.rs` | Native off-GPU coverage distinguishing malformed-RON parse errors from valid-RON-failing-validation (task 248) |

Four of the `renderer` test files above need `--all-features` (or at least
`--features animation`/`--features native`) to actually run, not just
`cargo test`: `gltf_animation_loader_test.rs` imports the
`animation`-feature-gated `renderer::webgl::animation` module (`src/webgl.rs`
line 26, `#[cfg(feature = "animation")]`); `gltf_attribute_descriptor_test.rs`,
`gltf_node_scene_test.rs`, and `gltf_skeleton_displacements_test.rs` are each
`#![cfg(all(feature = "native", not(target_arch = "wasm32")))]`-gated. All
four now carry a matching `required-features` `[[test]]` entry in
`Cargo.toml`, so a plain `cargo test` cleanly skips each with an explicit
"required features not available" notice instead of either failing to
compile or silently reporting "running 0 tests".
