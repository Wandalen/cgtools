# Renderer Tests

| File | Responsibility |
|------|----------------|
| manual/readme.md | Scripted browsee browser pixel-verification procedure (opaque path, webgpu/webgl) |
| webgl/node.rs | Tests Node structure functionality |
| webgl/scene.rs | Tests Scene structure functionality |
| webgl/camera.rs | Tests `Camera::new` parameter validation |
| webgl/shadow.rs | Tests `From<SpotLight> for Light` shadow-softness `size()` scaling |
| webgl/white_balance.rs | Tests `apply_white_balance` tint sign matches magenta/green direction |
| webgl/wide_outline.rs | Structural browser test: `WideOutlinePass` renders with the `outlineThickness` uniform wired up |
| webgl/jfa_step_size.rs | Tests the JFA step's real per-axis pixel jump is isotropic on a non-square canvas |
| webgl/jfa_silhouette.rs | Tests JFA init and outline-pass silhouette detection works for non-red object colors |
| webgl/outline_seed_sentinel.rs | Tests outline-pass JFA seed-validity check rejects sentinel, accepts zero coordinate |
| animation_tests.rs | Tests node animation system (transforms, rotation, scaling) |
| gltf_light_parsing_test.rs | Tests glTF light resolution and direction/position derivation |
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
| webgpu_geometry_test.rs | Tests `webgpu::Geometry::new` attribute-length cross-validation |
| gltf_loader_tests.rs | Verifies glTF loader asset-URI resolution rules |
| webgl_frame_orchestration_test.rs | Tests legacy webgl path's drawbuffers attachment selection |
