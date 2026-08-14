# Renderer Tests

| File | Responsibility |
|------|----------------|
| webgl/node.rs | Tests Node structure functionality |
| webgl/scene.rs | Tests Scene structure functionality |
| animation_tests.rs | Tests node animation system (transforms, rotation, scaling) |
| blender_tests.rs | Tests animation blending |
| color_grading_tests.rs | Tests color grading pipeline |
| geometry_tests.rs | Tests `Geometry` attribute API (add_attribute duplicate handling) |
| webgl/pbr_material.rs | Tests PBR material enums (`CullMode`, `AlphaMode`) — defaults, variants, clone/copy |
| scaler_tests.rs | Tests animation scaling |
| skeleton_tests.rs | Tests skeleton stuff |
| animation_graph_tests.rs | Tests animation graph stuff |
| mirror_tests.rs | Tests animation mirroring stuff |
| pmrem_tests.rs | Structural browser tests of the PMREM IBL generator |
| tests.rs | Connects test modules into root |
| shader_validation_tests.rs | Validates WGSL shader sources offline via naga |
| native_render_test.rs | Pixel-asserted opaque path render on the native backend |
| gltf_loader_tests.rs | Verifies glTF loader asset-URI resolution rules |
