> **Generated.** Do not edit manually. Maintained by `.locale.doc.generate`.
> Source of truth: `locales.config.yml` + `.persistent/locale.toml`.

# Locales — cgtools

A **locale** is a named, bounded directory representing a self-contained unit of development work. See [`willbe/locate/module/locate/docs/locale.md`](../../willbe/locate/module/locate/docs/locale.md) for the full specification.

All paths are relative to `~/pro/lib/wip_core/cgtools/dev`. `task` — Y = `task/` directory initialized.

## Summary

| # | rel-path | name | type | lang | purpose | task | last_active |
|---|----------|------|------|------|---------|------|-------------|
| 1 | `module/alias/browser_tools` | browser_tools | rust_crate | rs | Convenience alias crate for browser development tools | N | 2026-02-21 |
| 2 | `module/alias/ndarray_tools` | ndarray_tools | rust_crate | rs | Convenience alias crate for ndarray-based computer graphics | N | 2026-02-21 |
| 3 | `module/blank/cg_tools` | cg_tools | rust_crate | rs | Computer Graphics Toolkit | N | 2026-02-21 |
| 4 | `module/blank/cgtools` | cgtools | rust_crate | rs | Computer Graphics Toolkit | N | 2026-02-21 |
| 5 | `module/blank/mdmath` | mdmath | rust_crate | rs | Multidimensional math. | N | 2026-02-21 |
| 6 | `module/blank/mdmath_ai` | mdmath_ia | rust_crate | rs | Multidimensional math. | N | 2026-02-21 |
| 7 | `module/blank/mdmath_cg` | mdmath_cg | rust_crate | rs | Multidimensional math. | N | 2026-02-21 |
| 8 | `module/blank/mdmath_linalg` | mdmath_linalg | rust_crate | rs | Multidimensional math. | N | 2026-02-21 |
| 9 | `module/min/mingl` | mingl | rust_crate | rs | Minimal graphics library with abstract rendering backend | N | 2026-02-21 |
| 10 | `module/min/minwebgl` | minwebgl | rust_crate | rs | Minimal WebGL toolkit for concise graphics programming | N | 2026-02-21 |
| 11 | `module/min/minwebgpu` | minwebgpu | rust_crate | rs | Minimal WebGPU toolkit for modern graphics programming | N | 2026-02-21 |
| 12 | `module/min/minwgpu` | minwgpu | rust_crate | rs | — | N | 2026-02-21 |
| 13 | `module/helper/animation` | animation | rust_crate | rs | — | N | 2026-02-21 |
| 14 | `module/helper/browser_input` | browser_input | rust_crate | rs | Ergonomic input handling for WebAssembly applications | N | 2026-02-21 |
| 15 | `module/helper/browser_log` | browser_log | rust_crate | rs | Advanced logging and panic handling for WebAssembly applications | N | 2026-02-21 |
| 16 | `module/helper/canvas_renderer` | canvas_renderer | rust_crate | rs | 2D canvas renderer for WebGL applications with framebuffer support | N | 2026-02-21 |
| 17 | `module/helper/embroidery_tools` | embroidery_tools | rust_crate | rs | Tools for handling embroidery patterns, formats, and operations | N | 2026-02-21 |
| 18 | `module/helper/primitive_generation` | primitive_generation | rust_crate | rs | 3D geometry generation toolkit with primitives, text rendering | N | 2026-02-21 |
| 19 | `module/helper/line_tools` | line_tools | rust_crate | rs | High-performance line rendering for WebGL applications | N | 2026-02-21 |
| 20 | `module/helper/renderer` | renderer | rust_crate | rs | 3D renderer for WebGL applications with glTF loading | N | 2026-02-21 |
| 21 | `module/helper/tilemap_renderer` | tilemap_renderer | rust_crate | rs | Agnostic 2D rendering engine with backend adapter support | N | 2026-02-21 |
| 22 | `module/helper/tiles_tools` | tiles_tools | rust_crate | rs | High-performance tile-based game development toolkit | N | 2026-02-21 |
| 23 | `module/helper/behaviour_tree` | behaviour_tree | rust_crate | rs | — | N | 2026-02-21 |
| 24 | `module/helper/vectorizer` | vectorizer | rust_crate | rs | Raster-to-vector image conversion | N | 2026-02-21 |
| 25 | `module/math/mdmath_core` | mdmath_core | rust_crate | rs | Core multidimensional mathematics library | N | 2026-02-21 |
| 26 | `module/math/ndarray_cg` | ndarray_cg | rust_crate | rs | High-performance computer graphics mathematics library | N | 2026-02-21 |
| 27 | `examples/minwebgl/2d_line` | minwebgl_2d_line | rust_crate | rs | — | N | 2026-02-21 |
| 28 | `examples/minwebgl/3d_line` | minwebgl_3d_line | rust_crate | rs | — | N | 2026-02-21 |
| 29 | `examples/minwebgl/animation_amplitude_change` | animation_blending | rust_crate | rs | — | N | 2026-02-21 |
| 30 | `examples/minwebgl/animation_surface_rendering` | animation_surface_rendering | rust_crate | rs | — | N | 2026-02-21 |
| 31 | `examples/minwebgl/area_light` | area_light | rust_crate | rs | — | N | 2026-02-21 |
| 32 | `examples/minwebgl/attributes_instanced` | minwebgl_attributes_instanced | rust_crate | rs | — | N | 2026-02-21 |
| 33 | `examples/minwebgl/attributes_matrix` | minwebgl_attributes_matrix | rust_crate | rs | — | N | 2026-02-21 |
| 34 | `examples/minwebgl/attributes_vao` | minwebgl_attributes_vao | rust_crate | rs | — | N | 2026-02-21 |
| 35 | `examples/minwebgl/character_control` | character_control | rust_crate | rs | — | N | 2026-02-21 |
| 36 | `examples/minwebgl/color_space_conversions` | color_space_conversions | rust_crate | rs | — | N | 2026-02-21 |
| 37 | `examples/minwebgl/curve_surface_rendering` | curve_surface_rendering | rust_crate | rs | — | N | 2026-02-21 |
| 38 | `examples/minwebgl/deferred_shading` | deferred_shading | rust_crate | rs | — | N | 2026-02-21 |
| 39 | `examples/minwebgl/derive_tools_issue` | derive_tools_issue | rust_crate | rs | — | N | 2026-02-21 |
| 40 | `examples/minwebgl/diamond` | minwebgl_diamond | rust_crate | rs | — | N | 2026-02-21 |
| 41 | `examples/minwebgl/filter` | filter | rust_crate | rs | — | N | 2026-02-21 |
| 42 | `examples/minwebgl/filters` | filters | rust_crate | rs | — | N | 2026-02-21 |
| 43 | `examples/minwebgl/gltf_viewer` | minwebgl_gltf_viewer | rust_crate | rs | — | N | 2026-02-21 |
| 44 | `examples/minwebgl/hexagonal_grid` | hexagonal_grid | rust_crate | rs | — | N | 2026-02-21 |
| 45 | `examples/minwebgl/hexagonal_map` | hexagonal_map | rust_crate | rs | — | N | 2026-02-21 |
| 46 | `examples/minwebgl/jewelry_3d_site` | jewelry_3d_site | rust_crate | rs | — | N | 2026-02-21 |
| 47 | `examples/minwebgl/jewelry_site` | jewelry_site | rust_crate | rs | — | N | 2026-02-21 |
| 48 | `examples/minwebgl/lottie_surface_rendering` | lottie_surface_rendering | rust_crate | rs | — | N | 2026-02-21 |
| 49 | `examples/minwebgl/make_cube_map` | minwebgl_make_cube_map | rust_crate | rs | — | N | 2026-02-21 |
| 50 | `examples/minwebgl/mapgen_tiles_rendering` | mapgen_tiles_rendering | rust_crate | rs | — | N | 2026-02-21 |
| 51 | `examples/minwebgl/minimize_wasm` | minwebgl_minimize_wasm | rust_crate | rs | — | N | 2026-02-21 |
| 52 | `examples/minwebgl/morph_targets` | morph_targets | rust_crate | rs | — | N | 2026-02-21 |
| 53 | `examples/minwebgl/narrow_outline` | narrow_outline | rust_crate | rs | — | N | 2026-02-21 |
| 54 | `examples/minwebgl/obj_load` | minwebgl_obj_load | rust_crate | rs | — | N | 2026-02-21 |
| 55 | `examples/minwebgl/obj_viewer` | minwebgl_obj_viewer | rust_crate | rs | — | N | 2026-02-21 |
| 56 | `examples/minwebgl/object_picking` | object_picking | rust_crate | rs | — | N | 2026-02-21 |
| 57 | `examples/minwebgl/outline` | outline | rust_crate | rs | — | N | 2026-02-21 |
| 58 | `examples/minwebgl/pbr_lighting` | pbr_lighting | rust_crate | rs | — | N | 2026-02-21 |
| 59 | `examples/minwebgl/postprocessing` | postprocessing | rust_crate | rs | — | N | 2026-02-21 |
| 60 | `examples/minwebgl/raycaster` | raycaster | rust_crate | rs | — | N | 2026-02-21 |
| 61 | `examples/minwebgl/renderer_with_outlines` | renderer_with_outlines | rust_crate | rs | — | N | 2026-02-21 |
| 62 | `examples/minwebgl/shadowmap` | shadowmap | rust_crate | rs | — | N | 2026-02-21 |
| 63 | `examples/minwebgl/simple_pbr` | minwebgl_simple_pbr | rust_crate | rs | — | N | 2026-02-21 |
| 64 | `examples/minwebgl/skeletal_animation` | skeletal_animation | rust_crate | rs | — | N | 2026-02-21 |
| 65 | `examples/minwebgl/space_partition` | minwebgl_space_partition | rust_crate | rs | — | N | 2026-02-21 |
| 66 | `examples/minwebgl/spinning_cube_size_opt` | minwebgl_spinning_cube_size_opt | rust_crate | rs | — | N | 2026-02-21 |
| 67 | `examples/minwebgl/sprite_animation` | minwebgl_sprite_animation | rust_crate | rs | — | N | 2026-02-21 |
| 68 | `examples/minwebgl/text_msdf` | minwebgl_text_msdf | rust_crate | rs | — | N | 2026-02-21 |
| 69 | `examples/minwebgl/text_rendering` | text_rendering | rust_crate | rs | — | N | 2026-02-21 |
| 70 | `examples/minwebgl/trivial` | minwebgl_trivial | rust_crate | rs | — | N | 2026-02-21 |
| 71 | `examples/minwebgl/uniforms_animation` | minwebgl_uniforms_animation | rust_crate | rs | — | N | 2026-02-21 |
| 72 | `examples/minwebgl/uniforms_ubo` | minwebgl_uniforms_ubo | rust_crate | rs | — | N | 2026-02-21 |
| 73 | `examples/minwebgl/video_as_texture` | minwebgl_video_as_texture | rust_crate | rs | — | N | 2026-02-21 |
| 74 | `examples/minwebgl/wfc` | wfc | rust_crate | rs | — | N | 2026-02-21 |
| 75 | `examples/minwebgpu/deffered_rendering` | minwebgpu_deffered_rendering | rust_crate | rs | — | N | 2026-02-21 |
| 76 | `examples/minwebgpu/hello_triangle` | minwebgpu__ | rust_crate | rs | — | N | 2026-02-21 |
| 77 | `examples/minwgpu/grid_render` | grid_render | rust_crate | rs | — | N | 2026-02-21 |
| 78 | `examples/minwgpu/hello_triangle` | hello_triangle | rust_crate | rs | — | N | 2026-02-21 |
| 79 | `examples/math/life` | math_trivial | rust_crate | rs | — | N | 2026-02-21 |

---

## Profile

### workspace :: cgtools

| field | value |
|-------|-------|
| path | `lib/wip_core/cgtools/dev` |
| parent | `lib/wip_core` |
| type | rust_workspace |
| lang | rs |
| canonical | Y |
| task | Y |
| last_active | 2026-02-21 |

**Instances (2).** Both instances share the same git remote (`github.com/Wandalen/cgtools`).

| instance | path | branch | commit | last_active | notes |
|----------|------|--------|--------|-------------|-------|
| canonical | `lib/wip_core/cgtools/dev` | `master` | — | 2026-02-21 | Primary development worktree |
| lib_cgtools | `lib/cgtools` | `outline` | `21097922` | 2025-06-06 | External clone on `outline` branch |

**Purpose.** Computer graphics toolkit and example gallery. Core abstractions: `minwebgl`, `minwebgpu`, `minwgpu`. Helper modules for animation, rendering, tilemap, physics. Includes 70+ working WebGL/WGPU examples across 2D/3D rendering, skeletal animation, and PBR lighting.
