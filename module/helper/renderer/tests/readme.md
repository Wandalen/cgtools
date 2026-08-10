# Renderer Tests

| File | Responsibility |
|------|----------------|
| webgl/node.rs | Tests Node structure functionality |
| webgl/scene.rs | Tests Scene structure functionality |
| animation_tests.rs | Tests node animation system (transforms, rotation, scaling) |
| blender_tests.rs | Tests animation blending |
| color_grading_tests.rs | Tests color grading pipeline |
| geometry_tests.rs | Tests `Geometry` attribute API (add_attribute duplicate handling) |
| scaler_tests.rs | Tests animation scaling |
| skeleton_tests.rs | Tests skeleton stuff |
| animation_graph_tests.rs | Tests animation graph stuff |
| mirror_tests.rs | Tests animation mirroring stuff |
| tests.rs | Connects test modules into root |
| shader_validation_tests.rs | Validates WGSL shader sources offline via naga |
