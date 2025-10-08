# Examples

Interactive WebGL/WebGPU examples demonstrating CGTools capabilities.

## Quick Start

Prerequisites:
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Run example:
```bash
cd minwebgl/hexagonal_grid
trunk serve --release
# Open http://localhost:8080
```

## Math Examples

| | | |
|:-------------------------:|:-------------------------:|:-------------------------:|
|<img height="200" src="./math/life/showcase.png"><br>[Game of life](./math/life/readme.md) | | |

## WebGL Examples

| | | |
|:-------------------------:|:-------------------------:|:-------------------------:|
|<img height="200" src="./minwebgl/2d_line/showcase.png"><br>[2D line](./minwebgl/2d_line/readme.md) |<img height="200" src="./minwebgl/3d_line/showcase.png"><br>[3D line](./minwebgl/3d_line/readme.md) |<img height="200" src="./minwebgl/animation_surface_rendering/showcase.png"><br>[Animated objects surface rendering](./minwebgl/animation_surface_rendering/readme.md) |
|<img height="200" src="./minwebgl/area_light/showcase.png"><br>[Area light](./minwebgl/area_light/readme.md) |<img height="200" src="./minwebgl/attributes_instanced/showcase.png"><br>[Attributes instanced](./minwebgl/attributes_instanced/readme.md) |<img height="200" src="./minwebgl/attributes_matrix/showcase.png"><br>[Attributes matrix](./minwebgl/attributes_matrix/readme.md) |
|<img height="200" src="./minwebgl/attributes_vao/showcase.png"><br>[Attributes VAO](./minwebgl/attributes_vao/readme.md) |<img height="200" src="./minwebgl/color_space_conversions/showcase.png"><br>[Color space conversions](./minwebgl/color_space_conversions/readme.md) |<img height="200" src="./minwebgl/curve_surface_rendering/showcase.png"><br>[Curve rendering on surface](./minwebgl/curve_surface_rendering/readme.md)  |
|<img height="200" src="./minwebgl/deferred_shading/showcase.png"><br>[Deferred shading](./minwebgl/deferred_shading/readme.md) |<img height="200" src="./minwebgl/derive_tools_issue/showcase.png"><br>[Derive tools issue](./minwebgl/derive_tools_issue/readme.md) |<img height="200" src="./minwebgl/diamond/showcase.png"><br>[Diamond](./minwebgl/diamond/readme.md) |
|<img height="200" src="./minwebgl/filter/showcase.png"><br>[Image filter](./minwebgl/filter/readme.md) |<img height="200" src="./minwebgl/filters/showcase.png"><br>[Image filters](./minwebgl/filters/readme.md) |<img height="200" src="./minwebgl/gltf_viewer/showcase.png"><br>[GLTF viewer](./minwebgl/gltf_viewer/readme.md) |
|<img height="200" src="./minwebgl/hexagonal_grid/showcase.png"><br>[Hexagonal grid](./minwebgl/hexagonal_grid/readme.md) |<img height="200" src="./minwebgl/hexagonal_map/showcase.png"><br>[Hexagonal map](./minwebgl/hexagonal_map/readme.md) |<img height="200" src="./minwebgl/make_cube_map/showcase.png"><br>[Cube map](./minwebgl/make_cube_map/readme.md) |
|<img height="200" src="./minwebgl/mapgen_tiles_rendering/showcase.png"><br>[Tilemaps rendering](./minwebgl/mapgen_tiles_rendering/readme.md) |<img height="200" src="./minwebgl/minimize_wasm/showcase.png"><br>[Minimize wasm](./minwebgl/minimize_wasm/readme.md) |<img height="200" src="./minwebgl/narrow_outline/showcase.png"><br>[Narrow outline](./minwebgl/narrow_outline/readme.md) |
|<img height="200" src="./minwebgl/obj_load/showcase.png"><br>[OBJ loading](./minwebgl/obj_load/readme.md) |<img height="200" src="./minwebgl/obj_viewer/showcase.png"><br>[OBJ viewer](./minwebgl/obj_viewer/readme.md) |<img height="200" src="./minwebgl/object_picking/showcase.png"><br>[Object picking](./minwebgl/object_picking/readme.md) |
|<img height="200" src="./minwebgl/outline/showcase.png"><br>[Outline](./minwebgl/outline/readme.md) |<img height="200" src="./minwebgl/raycaster/showcase.png"><br>[Raycaster](./minwebgl/raycaster/readme.md) |<img height="200" src="./minwebgl/renderer_with_outlines/showcase.png"><br>[Outlines postprocessing](./minwebgl/renderer_with_outlines/readme.md) |
|<img height="200" src="./minwebgl/simple_pbr/showcase.png"><br>[Simple PBR](./minwebgl/simple_pbr/readme.md) |<img height="200" src="./minwebgl/spinning_cube_size_opt/showcase.png"><br>[Spinning cube](./minwebgl/spinning_cube_size_opt/readme.md) |<img height="200" src="./minwebgl/sprite_animation/showcase.png"><br>[Sprite animation](./minwebgl/sprite_animation/readme.md) |
|<img height="200" src="./minwebgl/text_msdf/showcase.png"><br>[Text MSDF](./minwebgl/text_msdf/readme.md) |<img height="200" src="./minwebgl/text_rendering/showcase.png"><br>[Text rendering](./minwebgl/text_rendering/readme.md) |<img height="200" src="./minwebgl/trivial/showcase.png"><br>[Trivial](./minwebgl/trivial/readme.md) |
|<img height="200" src="./minwebgl/uniforms_animation/showcase.png"><br>[Uniform animation](./minwebgl/uniforms_animation/readme.md) |<img height="200" src="./minwebgl/uniforms_ubo/showcase.png"><br>[Uniform UBO](./minwebgl/uniforms_ubo/readme.md) |<img height="200" src="./minwebgl/video_as_texture/showcase.png"><br>[Video as texture](./minwebgl/video_as_texture/readme.md) |
|<img height="200" src="./minwebgl/wfc/showcase.png"><br>[Wave function collapse](./minwebgl/wfc/readme.md) | | |

## WebGPU Examples

| | | |
|:-------------------------:|:-------------------------:|:-------------------------:|
|<img height="200" src="./minwebgpu/deffered_rendering/showcase.png"><br>[Deffered rendering](./minwebgpu/deffered_rendering/readme.md) |<img height="200" src="./minwebgpu/hello_triangle/showcase.png"><br>[Hello triangle](./minwebgpu/hello_triangle/readme.md) | |

## WGPU Examples

| | | |
|:-------------------------:|:-------------------------:|:-------------------------:|
|<img height="200" src="./minwgpu/grid_render/showcase.png"><br>[Grid renderer](./minwgpu/grid_render/readme.md) |<img height="200" src="./minwgpu/hello_triangle/showcase.png"><br>[Hello triangle](./minwgpu/hello_triangle/readme.md) | |

## Development

```bash
# Development mode
trunk serve

# Production build
trunk build --release

# Clean build
trunk clean && cargo clean
```

## Structure

```
example/
├── src/main.rs
├── Cargo.toml
├── index.html
└── assets/
```

## Troubleshooting

- Check browser console for errors
- Verify WebGL/WebGPU support
- Use `trunk serve --release` for performance testing
