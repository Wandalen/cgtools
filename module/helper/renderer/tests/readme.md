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
| webgl/mod.rs | Connects the `webgl` test modules; configures the binary for a browser run and gates the live-context suites |
| webgl/geometry_gl_lifecycle.rs | Live-context: `Geometry`'s `Drop` deletes its own VAO and leaves shared attribute / index buffers alone; a cloned `Primitive` shares its geometry |
| webgl/texture_gl_lifecycle.rs | Live-context: an owning `Texture` is deleted once after its last clone drops; a view deletes nothing; `load_from_path` uploads into its own texture, skips the upload once dropped and settles a failed load through `onerror` (the `load_from_path` tests need `test_internals`) |
| webgl/ibl_gl_lifecycle.rs | Live-context: `IBL` texture teardown and clone / double-free behaviour (`test_internals`) |
| webgl/renderer_gl_lifecycle.rs | Live-context: `Renderer::gl_resources_free`, resize buffer replacement, and the skybox `Texture` the renderer keeps (an owning skybox survives `resize()`, a view is never deleted, a sourceless one is not drawn) (`test_internals`) |
| webgl/skeleton_gl_lifecycle.rs | Live-context: `TransformsData` / `DisplacementsData` texture teardown, clone-drop safety and upload-after-clone in either drop order (`test_internals`) |
| webgl/shadow_gl_lifecycle.rs | Live-context: `ShadowBaker` framebuffer teardown and cull-face restoration (`test_internals`) |
| webgl/gbuffer_gl_lifecycle.rs | Live-context: `GBuffer` VAO / framebuffer / depth buffer / texture teardown (`test_internals`) |
| webgl/blend_gl_lifecycle.rs | Live-context: `BlendPass::gl_resources_free` program teardown (`test_internals`) |
| webgl/unreal_bloom_gl_lifecycle.rs | Live-context: `UnrealBloomPass` drop-without-explicit-free teardown (`test_internals`) |
| webgl/wide_outline_gl_lifecycle.rs | Live-context: `WideOutlinePass::gl_resources_free` owned-vs-borrowed teardown (`test_internals`) |
| animation_tests.rs | Tests node animation system (transforms, rotation, scaling) |
| gltf_light_parsing_test.rs | Tests glTF light resolution and direction/position derivation |
| gltf_animation_loader_test.rs | Tests glTF animation channel decode + vec3 tween-sequence building |
| blender_tests.rs | Tests animation blending |
| color_grading_tests.rs | Tests color grading pipeline |
| geometry_tests.rs | Tests `Geometry` attribute API (add_attribute duplicate handling) |
| webgl/pbr_material.rs | Tests PBR material enums (`CullMode`, `AlphaMode`) — defaults, variants, clone/copy |
| webgl/texture.rs | Tests `Texture`'s builder and `Default` both target `TEXTURE_2D` and make non-owning views |
| scaler_tests.rs | Tests animation scaling |
| skeleton_tests.rs | Tests skeleton stuff |
| animation_graph_tests.rs | Tests animation graph stuff |
| mirror_tests.rs | Tests animation mirroring stuff |
| pbr_shading_readback_test.rs | Pixel readback of the real PBR program on a lit quad: clearcoat, anisotropy and normal-map shading properties (`test_internals`) |
| ar_alpha_readback_test.rs | Pixel readback of the AR alpha path: `ToSrgbPass` alpha forwarding stays premultiplied, `ColorGradingPass` passes alpha through, `alpha_forward_set` deletes the program it replaces, an `is_occluder` material hides the opaque geometry behind it in `Renderer::render` ( several occluders, in any scene order ) and in `ScenePicker`'s id buffer, the colour mask is restored before the skybox and after the frame, and the whole chain ( renderer, ACES, colour grading, `ToSrgbPass` ) leaves an occluder fully transparent on the canvas |
| tonemapping_edge_readback_test.rs | Pixel readback of the ACES `ToneMappingPass` over real `Renderer` frames: no light fringe on an opaque MSAA silhouette, no ring around a transparent quad's edge, coverage alpha forwarded on every path |
| tangent_frame_test.rs | Pixel readback of `main.frag`'s tangent-less `getTBN`: frame columns point along glTF's tangent space |
| clearcoat_anisotropy_shader_tests.rs | Headless-browser shader-compilation tests for KHR_materials_clearcoat / KHR_materials_anisotropy, from the renderer's own source assembly (`test_internals`) |
| engraving_shader_tests.rs | Headless-browser shader-compilation tests for the USE_ENGRAVING relief/roughness/darkening block |
| engraving_config_tests.rs | Native tests for engraving_config.json parsing, SizingMode resolution and validation, and the shared mip-chain sizes |
| engraving_session_tests.rs | Browser tests for the engraving runtime: `build` errors and per-node material copies, `MAX_TEXTURE_SIZE`, the `set_text` font whitelist, mask-texture chain checks, canvas mip count, PHYSICAL vs HYBRID sizing |
| pmrem_tests.rs | Structural browser tests of the PMREM IBL generator |
| fbo_pass_cycle_test.rs | Live-context FBO pass-cycle tests for `ShadowMap`/`GBuffer` bind/render |
| pbr_material_live_test.rs | Live-context tests for `PbrMaterial` defines/IBL-flag/emission/clone logic |
| pbr_texture_units_test.rs | `PbrMaterial` texture-unit layout: disjoint units, fragment samplers within WebGL2's guaranteed 16 |
| gltf_material_extensions_test.rs | Native parsing of `KHR_materials_clearcoat` / `KHR_materials_anisotropy` JSON: defaults, `texCoord`, coat normal `scale`, rotation |
| gltf_meshopt_test.rs | Native checks for meshopt-compressed glTF: parsing and validating each view's `EXT_meshopt_compression` / `KHR_meshopt_compression` object ( exact messages for malformed fields, every allowed layout accepted, mode and filter names round-tripped, both extensions on one view refused ), the pre-decode layout, range and overlap checks ( each view against itself, and views against each other in `decodes_overlap_check`, whose sweep matches a pairwise oracle and scales to 100 000 views ), fallback-buffer detection, and validation letting both extensions and `KHR_mesh_quantization` through, required or not ( `test_internals` ) |
| gltf_meshopt_browser_test.rs | Browser tests for meshopt glTF: `views_decode` through meshoptimizer's JS decoder, and `gltf::load`'s meshopt rules (fallback buffer allocated not fetched, oversized fallbacks refused alone or together and before any fetch, fallback buffers referenced only by compressed views, a buffer with both extensions refused), each refusal checked for its reason |
| gltf_buffers_test.rs | Native checks for the loader's buffer stage: `cpu_buffers_needed` is false for a static mesh and true for a skin, a morph target, or an animation when the `animation` feature builds its reader ( `test_internals` ) |
| gltf_buffers_browser_test.rs | Browser tests for the buffer rules `gltf::load` applies to every asset: the GLB chunk is `buffers[ 0 ]` only, a later uri-less buffer is refused, and so is a uri-less buffer with no chunk |
| gltf_quantization_test.rs | Native checks for `KHR_mesh_quantization`: normalized position bounds dequantized to shader units, and `accessor_vec3_read` over every component type with the view's stride ( `test_internals` ) |
| gltf_quantization_browser_test.rs | Browser test: `gltf::load` of a normalized `i16` mesh hands the scene dequantized bounds, not the accessor's raw integers |
| tests.rs | Connects test modules into root |
| shader_validation_tests.rs | Validates WGSL shader sources offline via naga |
| legacy_glsl_shader_compile_test.rs | Compiles all 28 shipped legacy GLSL ES 3.00 `.vert`/`.frag` shaders, plus `to_srgb.frag`'s `FORWARD_ALPHA` variant, through a real headless WebGL2 context |
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
| gltf_tangent_defines_test.rs | Live `gltf::load` of a `data:` URI asset: a TANGENT attribute turns on `USE_TANGENTS` in the fragment stage; `load` rejects an unsupported `extensionsRequired` entry and accepts the supported ones |
| gltf_extensions_required_test.rs | Tests glTF loader rejects assets requiring unsupported extensions |
| gltf_openpbr_params_test.rs | Tests glTF loader's OpenPBR Surface ingestion — KHR_materials_* scalar/color factor carriers and their per-extension schema defaults |
| openpbr_mtlx_test.rs | Tests the MaterialX (`.mtlx`) OpenPBR Surface reader — ASWF example fixtures, defaults, multiple surfaces, error handling |
| openpbr_gltf_mapping_test.rs | Tests the glTF-carrier → `OpenPbrSurface` mapping — factor correspondences, fixed-IOR clearcoat, implicit fuzz weight, clamping |
| openpbr_usda_test.rs | Tests the `.usda` → `.mtlx` asset-reference resolver — OpenPBRShaderPlayground pattern, multiple refs per material, non-material ignoring |
| openpbr_runtime_mapping_test.rs | Tests the `OpenPbrSurface → PbrMaterial` runtime reduction — conditional carriers, IOR-1.5 elision, disabled-layer handling |
| openpbr_lobe_texture_test.rs | Headless-browser gating + shader-compile tests for the §3.1 textured lobe carriers ( sheen color/roughness, iridescence weight textures ), plus the thin-film-under-IBL path. **Not yet executed** — needs `cargo test --target wasm32-unknown-unknown` with a browser driver |
| openpbr_transmission_test.rs | Tests the pure transmission math shared with `main.frag` — Beer's-law transmittance ( `mu_t = -ln( T ) / lambda` ), slab traversal length, roughness blur LOD, and the `transmission_depth` → attenuation-distance / thin-walled → zero-thickness carrier mapping |
| openpbr_transmission_shader_test.rs | Headless-browser gating + shader-compile tests for the three transmission variants ( slab, Beer-Lambert absorption, thin-walled ). **Not yet executed** — needs `cargo test --target wasm32-unknown-unknown` with a browser driver |
| openpbr_sheen_albedo_shader_test.rs | Headless-browser shader-compile tests for the fuzz albedo-scaling path ( `USE_SHEEN_ALBEDO` on, off, and composed with `USE_IBL` / `USE_KULLA_CONTY` ). **Not yet executed** — needs `cargo test --target wasm32-unknown-unknown` with a browser driver |
| usd_scene_test.rs | Tests the `loaders::usd` N3 core — in-memory `ar::Resolver` stage open, fan triangulation, primvar corner resolution ( vertex / faceVarying / indexed ), `Xform` world-matrix composition, malformed-topology error paths, the pure `usd_scene_analyze` scene pass ( hierarchy, shading-subtree skip, material binding + inheritance, visibility/purpose, matrix bridging ) and the `.mtlx`-bound material lane via `UsdAssetProvider`. Feature-gated `native-formats`. The GL half ( `usd_scene_load` ) was **browser-verified** — via the `gltf_viewer` USD scene mode |
| kulla_conty_test.rs | Tests the Kulla–Conty `E(μ,α)` / `E_avg(α)` table generator — bounds, energy conservation, roughness monotonicity — the per-bounce Fresnel series `F_ms` weighting it ( mirror / black limits, never exceeding the single-bounce weight, tint saturation ), and the anisotropic effective roughness the table is indexed by ( isotropic identity, geometric mean, monotonicity, degenerate axes ) |
| openpbr_specular_weight_test.rs | Proves `specular_weight` applied as an `F0` factor is *identical* to the spec’s adjusted-IOR construction ( `F0’ = eps² = xi·F_s` ), including below IOR 1 where the `sgn` term flips — guards against a redundant adjusted-IOR path being added |
| sheen_albedo_test.rs | Tests the fuzz ( sheen ) directional-albedo table generator — bounds, grazing-to-head-on falloff, the shared `0.1` roughness floor, lobe widening with roughness, determinism and the LUT layout |
| unreal_bloom_tests.rs | Structural browser tests: `UnrealBloomPass` renders via the real `SwapFramebuffer`-bound pass cycle |
