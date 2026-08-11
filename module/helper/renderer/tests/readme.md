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
| clearcoat_anisotropy_shader_tests.rs | Headless-browser shader-compilation tests for KHR_materials_clearcoat / KHR_materials_anisotropy |
| engraving_shader_tests.rs | Headless-browser shader-compilation tests for the USE_ENGRAVING relief/roughness/darkening block |
| engraving_config_tests.rs | Native tests for engraving_config.json parsing, SizingMode resolution and validation |
| pmrem_tests.rs | Structural browser tests of the PMREM IBL generator |
| tests.rs | Connects test modules into root |
| shader_validation_tests.rs | Validates WGSL shader sources offline via naga |
| native_render_test.rs | Pixel-asserted opaque path render on the native backend |
