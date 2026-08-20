# Renderer Tests

| File | Responsibility |
|------|----------------|
| manual/readme.md | Scripted browsee browser pixel-verification procedure (opaque path, webgpu/webgl) |
| webgl/node.rs | Tests Node structure functionality |
| webgl/mesh.rs | Tests `Mesh::clone` gives the clone an independent `Skeleton` Rc |
| webgl/scene.rs | Tests Scene structure functionality |
| webgl/camera.rs | Tests `Camera::new`/`projection_matrix_set` validation |
| webgl/shadow.rs | Tests `From<SpotLight> for Light` shadow-softness `size()` scaling |
| webgl/white_balance.rs | Tests `apply_white_balance` tint sign matches magenta/green direction |
| webgl/gbuffer.rs | Tests `GBufferAttachment::define_const`/`attribute_info` mapping correctness |
| webgl/wide_outline.rs | Structural browser test: `WideOutlinePass` renders with the `outlineThickness` uniform wired up |
| webgl/jfa_step_size.rs | Tests the JFA step's real per-axis pixel jump is isotropic on a non-square canvas |
| webgl/jfa_silhouette.rs | Tests JFA init and outline-pass silhouette detection works for non-red object colors |
| webgl/outline_seed_sentinel.rs | Tests outline-pass JFA seed-validity check rejects sentinel, accepts zero coordinate |
| webgl/jfa_buffer_selection.rs | Tests JFA ping-pong buffer selection matches the last step actually rendered |
| webgl/vibrance.rs | Tests `adjust_vibrance` saturation-push weight decreases with existing saturation |
| webgl/displacement_texture_size.rs | Tests `displacement_texture_size_compute` never collapses row width to zero |
| webgl/program_needs_recompile.rs | Tests `program_needs_recompile` invalidates a material's cached program on IBL-state change |
| webgl/pass.rs | Tests `SwapFramebuffer::new`'s doc comment renderbuffer claim matches its body |
| webgl/ibl.rs | Tests `ibl_texture_parameters_apply` targets `specular_1_texture`'s mip range, not `diffuse_texture`'s |
| animation_tests.rs | Tests node animation system (transforms, rotation, scaling) |
| gltf_light_parsing_test.rs | Tests glTF light resolution and direction/position derivation |
| gltf_animation_loader_test.rs | Tests glTF animation channel decode + vec3 tween-sequence building |
| blender_tests.rs | Tests animation blending |
| color_grading_tests.rs | Tests color grading pipeline |
| geometry_tests.rs | Tests `Geometry` attribute API (add_attribute duplicate handling) |
| webgl/pbr_material.rs | Tests PBR material enums (`CullMode`, `AlphaMode`) — defaults, variants, clone/copy |
| scaler_tests.rs | Tests animation scaling |
| skeleton_tests.rs | Tests skeleton stuff |
| animation_graph_tests.rs | Tests animation graph stuff |
| mirror_tests.rs | Tests animation mirroring stuff |
| pmrem_tests.rs | Structural browser tests of the PMREM IBL generator |
| fbo_pass_cycle_test.rs | Live-context FBO pass-cycle tests for `ShadowMap`/`GBuffer` bind/render |
| pbr_material_live_test.rs | Live-context tests for `PbrMaterial` defines/IBL-flag/emission/clone logic |
| tests.rs | Connects test modules into root |
| shader_validation_tests.rs | Validates WGSL shader sources offline via naga |
| legacy_glsl_shader_compile_test.rs | Compiles all 28 shipped legacy GLSL ES 3.00 `.vert`/`.frag` shaders through a real headless WebGL2 context |
| native_render_test.rs | Pixel-asserted opaque path render on the native backend |
| webgpu_geometry_test.rs | Tests `webgpu::Geometry::new` attribute-length cross-validation |
| webgpu_light_test.rs | Tests `webgpu::Lights::spot_push` cone-angle validation |
| webgpu_normal_matrix_test.rs | Tests `webgpu::normal_matrix_compute` singular-matrix identity fallback |
| gltf_loader_tests.rs | Verifies glTF loader asset-URI resolution rules |
| gltf_material_variation_test.rs | Tests glTF material-variation cache lookup-or-insert sharing |
| gltf_node_scene_test.rs | Tests glTF node-hierarchy, skeleton-attach, and scene-assembly builders |
| gltf_skeleton_displacements_test.rs | Tests glTF morph-target displacement data packing |
| gltf_attribute_descriptor_test.rs | Tests glTF vertex-attribute descriptor computation from accessor metadata |
| webgl_frame_orchestration_test.rs | Tests legacy webgl path's drawbuffers attachment selection |
| webgl_renderer_pass_cycle_test.rs | Live-context test: legacy `Renderer::render()` completes on an opaque PBR primitive and an empty scene |
| gltf_extensions_required_test.rs | Tests glTF loader rejects assets requiring unsupported extensions |
| unreal_bloom_tests.rs | Structural browser tests: `UnrealBloomPass` renders via the real `SwapFramebuffer`-bound pass cycle |
