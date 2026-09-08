# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- **USD scene ingestion core ( N3 first slice, browser-untested )**: new optional `native-formats` feature ( `openusd` + `openusd-schemas` `geom`/`shade`, pure Rust, compiles for wasm - spike recorded in the adoption plan §2.2 ). `loaders::usd` ships the off-GPU core : `UsdInMemoryResolver` ( an `ar::Resolver` over a `path -> bytes` map, the browser feed for HTTP-fetched `.usda`/`.mtlx`/`.usdz` bytes with no filesystem ), `usd_stage_open`, `usd_mesh_extract` ( `UsdGeomMesh` fan triangulation with vertex/faceVarying/indexed primvar corner resolution for `normals` / `primvars:st` / `primvars:displayColor`, erroring - never panicking - on malformed topology ) and `usd_local_to_world` ( ancestor `Xform` composition ). Native-tested in `tests/usd_scene_test.rs` ( 9 cases, inline `.usda` fixtures served through the resolver ). GL scene assembly and material binding resolve onto this core as the next slice.
- **OpenPBR Surface ingestion (loader stage)**: the glTF loader now reads the scalar/color factors of the `KHR_materials_*` extensions that transport ASWF OpenPBR Surface parameters (`KHR_materials_ior`, `KHR_materials_sheen`, `KHR_materials_transmission`, `KHR_materials_volume`, `KHR_materials_iridescence`, `KHR_materials_emissive_strength`, `KHR_materials_dispersion`, `KHR_materials_diffuse_transmission`) into the new `PbrMaterial::openpbr_params` field (`OpenPbrParams`), via the pure off-GPU `loaders::gltf::material_openpbr_params_read` (native-tested in `gltf_material_extensions_test.rs`). Parsed-and-stored only: the lobes' shader evaluation and texture maps are deferred follow-ups, so rendering behavior is unchanged.
- **OpenPBR Surface shading (opaque path)**: a material carrying any OpenPBR-carrying extension now selects the spec's opaque layered evaluation in `main.frag` behind the `USE_OPENPBR` define — IOR-driven dielectric Fresnel (`specular_ior` → F0), the energy-preserving per-axis `alpha_t`/`alpha_b` mapping plus Smith joint-visibility GGX (per the OpenPBR spec's microfacet model), a fuzz/sheen lobe (`KHR_materials_sheen`, Charlie NDF + Ashikhmin-Premoze visibility, direct + IBL), and KHR_materials_emissive_strength scaling (also enabled on the legacy path). Materials without those extensions keep the legacy glTF metallic-roughness shader unchanged. Refraction (transmission / volume / subsurface), thin-film iridescence, dispersion and the textured lobe carriers remain deferred.
- **MaterialX (`.mtlx`) OpenPBR lane**: the new canonical `OpenPbrSurface` parameter model (full spec parameter set with spec defaults, `material/openpbr_surface.rs`) and the pure, off-GPU `loaders::openpbr_mtlx::openpbr_surfaces_from_mtlx` reader map ASWF `open_pbr_*.mtlx` surfaces onto it — native-tested in `tests/openpbr_mtlx_test.rs` against verbatim `open_pbr_default/gold/glass/velvet.mtlx` fixtures. Reuses the workspace `quick-xml` dependency.
- **glTF → `OpenPbrSurface` bridge**: `openpbr_from_gltf` maps the core `pbrMetallicRoughness` + `KHR_materials_*` scalar carriers (base color/metalness/roughness, specular weight/color/ior/anisotropy, clearcoat, sheen→fuzz, transmission) onto the canonical surface; lobes glTF cannot express keep their spec defaults. Native-tested in `tests/openpbr_gltf_mapping_test.rs`.
- **USD `.usda` → `.mtlx` resolver (N3-lite)**: `loaders::openpbr_usda::usda_mtlx_references` scans `Material` prims for `references = @….mtlx@…` asset pointers (the OpenPBR content pattern, e.g. OpenPBRShaderPlayground) so the N2 `.mtlx` reader can take over; inline/nodegraph USD stays on the future `openusd` Stage reader. Native-tested in `tests/openpbr_usda_test.rs` and verified end-to-end against the real playground `iceCube.usda` + its `.mtlx`.
- **`OpenPbrSurface → PbrMaterial` runtime bridge**: `openpbr_to_runtime` reduces a canonical surface to glTF-shaped runtime factors (base RGBA/metalness/roughness, KHR specular IOR/weight/color, clearcoat→coat, fuzz) and `PbrMaterial::openpbr_surface_apply` applies them through the `USE_OPENPBR` path — the missing direction that lets `.mtlx`/`.usda`-loaded materials actually render. Native-tested in `tests/openpbr_runtime_mapping_test.rs`.
- **Thin-film iridescence (opaque path)**: `USE_OPENPBR_IRIDESCENCE` evaluates the analytic spectral thin-film Fresnel (`evalIridescence`, Airy-derivative with CIE sensitivity fits) and mixes it into `F_s` by the iridescence weight; carries OpenPBR `thin_film_*` through the `KHR_materials_iridescence` carriers. Note the reference model's degenerate case: on a near-perfect mirror substrate ( e.g. metal gold ) the interference terms cancel and the film only lifts `F_s` toward 1 - visible iridescence needs low-reflectance bases.
- **Kulla-Conty multi-scatter energy compensation (direct lights)**: `loaders::kulla_conty` precomputes the directional-albedo LUT `E( mu, roughness )` + row-constant `E_avg( roughness )` ( GGX VNDF importance sampling, native-tested in `kulla_conty_test.rs` ), the `Renderer` uploads/binds it as an RGBA32F texture, and `main.frag` adds `f_ms = (1-Eo)(1-Ei) / ( PI (1-Eavg) ) * Favg * irradiance` per direct light behind `USE_KULLA_CONTY` for `USE_OPENPBR` materials. IBL keeps its existing Fdez-Agera split-sum compensation.
- **Textured lobe carriers (subset, browser-untested)**: `PbrMaterial` gains `sheen_color_texture` / `sheen_roughness_texture` ( `KHR_materials_sheen`, OpenPBR `fuzz_color`/`fuzz_roughness` ) and `iridescence_texture` ( `KHR_materials_iridescence`, `thin_film_weight` ) slots; texture presence alone gates `USE_OPENPBR` / `USE_OPENPBR_IRIDESCENCE`, the loader resolves the `TextureInfo` blocks, and the shader multiplies factors by RGB / A channels ( glTF factor-defaults-to-one semantics ). Fragment sampler units 20-22. The remaining §3.1 carriers ( transmission / thickness / iridescenceThickness / diffuseTransmission textures ) are deferred to the §3.3 transmission pass. Native loader tests in `gltf_material_extensions_test.rs` pass; the browser gating/compile suite `openpbr_lobe_texture_test.rs` is written but not yet executed ( needs `cargo test --target wasm32-unknown-unknown` + a browser driver ).

### Changed

- **`specular_ior` decoupled from the OpenPBR surface path**: an `KHR_materials_ior`-only material now raises just `USE_OPENPBR_IOR` ( the IOR-driven F0 swap ) instead of the full `USE_OPENPBR` evaluation; the whole path is reserved for materials carrying OpenPBR-only lobes ( fuzz / thin-film / transmission / ... ).
- **glTF loader OpenPBR params go through the setter**: `materials_create` previously assigned `openpbr_params` directly, bypassing the defines cache - materials whose last define-raising setter ran *before* the assignment compiled without `USE_OPENPBR`. Also applies the glTF rule that a factor omitted while its texture is present defaults to full strength.

- **IBL multiple-scattering energy compensation**: indirect specular now adds the multi-scatter term (`Fms * Ems` weighted by irradiance) on top of the single-scatter prefiltered reflection, matching three.js `computeMultiscattering()`. Without it, rough metals/plastics read as pure mirrors and the overall specular is too dim.
- **Exposure applied uniformly**: `Renderer::set_exposure` now scales the entire lit result in the PBR shader (`color *= exp2( exposure )`) instead of only the IBL contribution. Previously exposure multiplied just the environment term, over-brightening reflections relative to direct lighting.
- **ACES pre-exposure scaling**: the ACES tone mapping pass now divides by `0.6` before the RRT fit, matching three.js `ACESFilmicToneMapping` so identical exposure values produce identical brightness.
- **Shader program caching**: Materials sharing identical shader source code now reuse a single compiled GPU program instead of compiling duplicates. Programs are keyed by `(TypeId, defines_string)` in a `shader_source_registry` — materials of the same concrete Rust type with identical defines always produce the same shader source, so one compiled program is shared across all instances.
- **Draw call grouping**: Opaque and transparent primitives are sorted by program UUID before drawing, minimizing GPU state switches (program binds).
- **Three-phase rendering pipeline**: The render loop is now split into (1) scene traversal & program compilation, (2) per-program uniform uploads (camera, lights, exposure), and (3) sorted draw calls.
- **Material `bind()` contract**: `bind()` is now the single method responsible for activating texture units, uploading texture data, and binding textures. Implementations must call `gl.active_texture()` before each texture bind. The `upload_textures()` trait method has been removed.
- **IBL texture safety**: IBL textures are rebound after every `material.bind()` call, preventing non-IBL materials from accidentally overwriting IBL texture units.
- **Dirty-flag pattern for `needs_update`**: `PbrMaterial::needs_update` is now `Cell<bool>` with interior mutability. The renderer calls `set_needs_update(false)` after uploading uniforms, so `upload_on_state_change()` is skipped on subsequent frames unless the material is explicitly marked dirty via `set_needs_update(true)`.
- **BREAKING**: Renamed `shadow::Light::light_size()` to `shadow::Light::size()`
- Upgraded shadow depth format from `DEPTH_COMPONENT24` to `DEPTH_COMPONENT32F`
- **BREAKING**: Renamed all `get_`-prefixed `Material` trait methods: `get_id` → `id`, `get_name` → `name`, `get_needs_update` → `needs_update`, `get_ibl_base_texture_unit` → `ibl_base_texture_unit`, `get_vertex_shader` → `vertex_shader`, `get_fragment_shader` → `fragment_shader`, `get_defines_str` → `defines_str`, `get_vertex_defines_str` → `vertex_defines_str`, `get_fragment_defines_str` → `fragment_defines_str`, `get_alpha_mode` → `alpha_mode`, `get_cull_mode` → `cull_mode`, `get_front_face` → `front_face`, `get_depth_func` → `depth_func`, `get_color_write_mask` → `color_write_mask`
- **BREAKING**: Renamed `Material::set_compiled()` to `Material::clear_recompile_flag()`
- **BREAKING**: `Material::set_needs_update()` is now a required method (no default no-op implementation)
- **BREAKING**: Renamed `PbrMaterial::get_vertex_defines()` → `vertex_defines()`, `get_fragment_defines()` → `fragment_defines()`
- **BREAKING**: Renamed `Renderer::get_exposure()` → `exposure()`, `get_bloom_radius()` → `bloom_radius()`, `get_bloom_strength()` → `bloom_strength()`, `get_main_texture()` → `main_texture()`
- **BREAKING**: Renamed `GBuffer::get_texture()` → `texture()`
- **BREAKING**: Removed `shader_hash()` from the `Material` trait (dead code, replaced by `(TypeId, defines_str)` cache key)
- **BREAKING**: Asset loaders (`webgl::loaders::gltf::load`, `webgl::loaders::ibl::load`, `webgl::loaders::hdr_texture::load_to_mip_cube` / `load_to_mip_d2`) no longer rely on `mingl::file::load`'s implicit `/static/` prefix. Path arguments are now passed verbatim to the underlying fetch — callers that previously passed bare paths like `"envMap"` must now pass `"static/envMap"` (or any other valid URL / origin-absolute path). Migration mirrors the upstream `mingl` 0.4.0 change.
- **BREAKING**: `Renderer::set_use_emission` now takes a `&WebGl2RenderingContext` as its first parameter (`set_use_emission( &mut self, gl, use_emission )`). The context is needed to lazily allocate the bloom pass and swap framebuffer the first time emission is enabled.

### Fixed

- **Kulla-Conty gating substring bug**: `Renderer::primitive_register` decided to bind the Kulla-Conty LUT with `defines.contains( "USE_OPENPBR" )`, which substring-matches `USE_OPENPBR_IOR` — an IOR-only material silently got the multi-scatter energy compensation ( visible as gold "turning diffuse-bright" when touching the IOR slider ). Now matches the exact `#define USE_OPENPBR` line.
- **Screen-space pass culling**: all post-processing passes (tonemapping, sRGB, bloom, color-grading, blend, shadow-to-color) and the OIT composite now explicitly call `gl.disable(CULL_FACE)` before drawing the fullscreen triangle. The fullscreen triangle is back-facing from the camera's perspective, so any preceding opaque pass that leaves `CULL_FACE` enabled would silently cull it, producing a black frame.
- **Bloom alpha channel corruption**: `unreal_bloom.frag` now writes `alpha = 0.0` instead of `1.0`. The main framebuffer alpha channel is used to distinguish geometry pixels (alpha `1`) from background (alpha `0`) for tone mapping and subsequent passes. Writing alpha `1` from the additive bloom blit was overwriting that signal.
- Clear-color background is no longer affected by exposure or tone mapping. The main color target is cleared with alpha `0` to mark background pixels (geometry and skybox write alpha `1`), and the tone mapping pass leaves alpha-`0` pixels untouched — mirroring three.js, where the clear color bypasses tone mapping.
- Removed leftover `format!( "static/{}/{}", ... )` in `webgl::loaders::gltf::load`'s texture-Uri branch which, after the `mingl::file::load` semantics change, produced `static/static/<path>` URLs for any glTF with external textures.
- Fixed IBL texture corruption where `upload_textures()` could overwrite IBL texture units because `active_texture` was not reset after `ibl.bind()`.
- Fixed `light_map` texture not being bound in `PbrMaterial::bind()` (was missing from the bind list).
- Fixed texture unit state leak in custom materials (`GemMaterial`, `SurfaceMaterial`) — `upload()` is now called inside `bind()` with explicit `active_texture()` per unit.
- Fixed `AlphaMode::Mask` materials incorrectly routed to WBOIT transparent pass. Mask uses binary alpha cutoff and needs depth writes, which WBOIT disables. Now routed to opaque pass.
- Fixed off-by-one in light upload bounds check (`i > MAX_*_LIGHTS` → `i >= MAX_*_LIGHTS`). Index 8 is out of bounds for shader arrays declared as `lights[8]`.
- Fixed non-deterministic shader cache keys caused by `FxHashMap` iteration order in `rebuild_defines_cache()`. Entries are now sorted alphabetically before building the defines string.
- **BREAKING**: `PbrMaterial` texture fields (`base_color_texture`, `metallic_roughness_texture`, `normal_texture`, `occlusion_texture`, `emissive_texture`, `specular_texture`, `specular_color_texture`, `light_map`) are now private. Use setter methods (e.g. `set_base_color_texture()`) which automatically call `rebuild_defines_cache()`.

### Removed

- Removed `upload_textures()` from the `Material` trait.
- Removed `base_shader_hash()` from the `Material` trait and `PbrMaterial`.
- Removed dead/commented-out rendering code from `renderer.rs`.

### Added

- GPU PMREM generation (`webgl::loaders::pmrem::generate`): converts an equirectangular HDR into a full IBL set — equirect→cubemap, GGX importance-sampled prefiltered specular mips, cosine-weighted irradiance convolution, and a split-sum BRDF integration LUT.
- `cull_mode` field to `PbrMaterial` for fine-grained face culling control
- `Drop` implementation for `SwapFramebuffer` to prevent GPU memory leaks
- GSAA (Geometric Specular Anti-Aliasing) for improved specular highlights
- Reflection-space LOD bias for reduced IBL aliasing
- Dither noise (IGN) for HDR banding reduction
- Firefly suppression via selective Reinhard tonemapping
- `highp` precision qualifiers for IBL and shadow map samplers

## [0.1.0] - 2024-08-08

### Added

- Initial release of renderer crate
- 3D renderer for WebGL applications
- glTF model loading and processing support
- Scene graph management with hierarchical transforms
- Material system with PBR support
- Mesh rendering with vertex/index buffers
- Camera controls and projection management
- Post-processing pipeline with multiple effects
- Outline rendering (narrow and wide variants)
- Image-based lighting (IBL) support
- Texture and sampler management
- WebAssembly-optimized rendering pipeline

[0.1.0]: https://github.com/Wandalen/cgtools/releases/tag/renderer-v0.1.0
