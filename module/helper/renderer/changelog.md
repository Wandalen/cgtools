# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Changed

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
