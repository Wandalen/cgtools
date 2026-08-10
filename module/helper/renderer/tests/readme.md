# Renderer Tests

| File | Responsibility |
|------|----------------|
| webgl/node.rs | Tests Node structure functionality |
| webgl/scene.rs | Tests Scene structure functionality |
| animation_test.rs | Tests node animation system (transforms, rotation, scaling) |
| blender_tests.rs | Tests animation blending |
| color_grading_test.rs | Tests color grading pipeline |
| scaler_tests.rs | Tests animation scaling |
| skeleton_tests.rs | Tests skeleton stuff |
| animation_graph_tests.rs | Tests animation graph stuff |
| mirror_tests.rs | Tests animation mirroring stuff |
| clearcoat_anisotropy_shader_tests.rs | Headless-browser shader-compilation tests for KHR_materials_clearcoat / KHR_materials_anisotropy |
| engraving_shader_tests.rs | Headless-browser shader-compilation tests for the USE_ENGRAVING relief/roughness/darkening block |
| engraving_config_tests.rs | Native tests for engraving_config.json parsing, SizingMode resolution and validation |
| tests.rs | Connects test modules into root |
