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

More detailed setup and run instruction: [how_to_run.md](./how_to_run.md)

## Math Examples

| | | |
|:-------------------------:|:-------------------------:|:-------------------------:|
|<img width="500px" src="./math/life/showcase.png"><br>[Game of life](./math/life/readme.md) | | |

## WebGL Examples

| | | |
|:-------------------------:|:-------------------------:|:-------------------------:|
|[2D line](./minwebgl/2d_line/readme.md)<br>*(No showcase yet)* |<img width="500px" src="./minwebgl/3d_line/showcase.gif"><br>[3D line](./minwebgl/3d_line/readme.md) |<img width="500px" src="./minwebgl/animation_surface_rendering/showcase.png"><br>[Animated objects surface rendering](./minwebgl/animation_surface_rendering/readme.md) |
|<img width="500px" src="./minwebgl/area_light/showcase.png"><br>[Area light](./minwebgl/area_light/readme.md) |<img width="500px" src="./minwebgl/attributes_instanced/showcase.png"><br>[Attributes instanced](./minwebgl/attributes_instanced/readme.md) |<img width="500px" src="./minwebgl/attributes_matrix/showcase.gif"><br>[Attributes matrix](./minwebgl/attributes_matrix/readme.md) |
|<img width="500px" src="./minwebgl/attributes_vao/showcase.png"><br>[Attributes VAO](./minwebgl/attributes_vao/readme.md) |<img width="500px" src="./minwebgl/color_space_conversions/showcase.png"><br>[Color space conversions](./minwebgl/color_space_conversions/readme.md) |<img width="500px" src="./minwebgl/curve_surface_rendering/showcase.png"><br>[Curve rendering on surface](./minwebgl/curve_surface_rendering/readme.md) |
|<img width="500px" src="./minwebgl/deferred_shading/showcase.png"><br>[Deferred shading](./minwebgl/deferred_shading/readme.md) |<img width="500px" src="./minwebgl/diamond/showcase.gif"><br>[Diamond](./minwebgl/diamond/readme.md) |<img width="500px" src="./minwebgl/filter/showcase.png"><br>[Image filter](./minwebgl/filter/readme.md) |
|<img width="500px" src="./minwebgl/filters/showcase.gif"><br>[Image filters](./minwebgl/filters/readme.md) |<img width="500px" src="./minwebgl/gltf_viewer/showcase.jpg"><br>[GLTF viewer](./minwebgl/gltf_viewer/readme.md) |<img width="500px" src="./minwebgl/hexagonal_grid/showcase.png"><br>[Hexagonal grid](./minwebgl/hexagonal_grid/readme.md) |
|<img width="500px" src="./minwebgl/hexagonal_map/showcase.png"><br>[Hexagonal map](./minwebgl/hexagonal_map/readme.md) |<img width="500px" src="./minwebgl/make_cube_map/showcase.jpg"><br>[Cube map](./minwebgl/make_cube_map/readme.md) |<img width="500px" src="./minwebgl/mapgen_tiles_rendering/showcase.png"><br>[Tilemaps rendering](./minwebgl/mapgen_tiles_rendering/readme.md) |
|<img width="500px" src="./minwebgl/minimize_wasm/showcase.gif"><br>[Minimize wasm](./minwebgl/minimize_wasm/readme.md) |<img width="500px" src="./minwebgl/narrow_outline/showcase.png"><br>[Narrow outline](./minwebgl/narrow_outline/readme.md) |<img width="500px" src="./minwebgl/obj_load/showcase.gif"><br>[OBJ loading](./minwebgl/obj_load/readme.md) |
|<img width="500px" src="./minwebgl/obj_viewer/showcase.png"><br>[OBJ viewer](./minwebgl/obj_viewer/readme.md) |<img width="500px" src="./minwebgl/object_picking/showcase.gif"><br>[Object picking](./minwebgl/object_picking/readme.md) |<img width="500px" src="./minwebgl/outline/showcase.png"><br>[Outline](./minwebgl/outline/readme.md) |
|<img width="500px" src="./minwebgl/raycaster/showcase.png"><br>[Raycaster](./minwebgl/raycaster/readme.md) |<img width="500px" src="./minwebgl/renderer_with_outlines/showcase.png"><br>[Outlines postprocessing](./minwebgl/renderer_with_outlines/readme.md) |<img width="500px" src="./minwebgl/simple_pbr/showcase.gif"><br>[Simple PBR](./minwebgl/simple_pbr/readme.md) |
|<img width="500px" src="./minwebgl/spinning_cube_size_opt/showcase.gif"><br>[Spinning cube](./minwebgl/spinning_cube_size_opt/readme.md) |<img width="500px" src="./minwebgl/sprite_animation/showcase.gif"><br>[Sprite animation](./minwebgl/sprite_animation/readme.md) |<img width="500px" src="./minwebgl/text_msdf/showcase.jpg"><br>[Text MSDF](./minwebgl/text_msdf/readme.md) |
|<img width="500px" src="./minwebgl/text_rendering/showcase.png"><br>[Text rendering](./minwebgl/text_rendering/readme.md) |<img width="500px" src="./minwebgl/trivial/showcase.png"><br>[Trivial](./minwebgl/trivial/readme.md) |<img width="500px" src="./minwebgl/uniforms_animation/showcase.gif"><br>[Uniform animation](./minwebgl/uniforms_animation/readme.md) |
|<img width="500px" src="./minwebgl/video_as_texture/showcase.gif"><br>[Video as texture](./minwebgl/video_as_texture/readme.md) |<img width="500px" src="./minwebgl/wfc/showcase.png"><br>[Wave function collapse](./minwebgl/wfc/readme.md) |

## WebGPU Examples

| | | |
|:-------------------------:|:-------------------------:|:-------------------------:|
|<img width="500px" src="./minwebgpu/deffered_rendering/showcase.jpg"><br>[Deffered rendering](./minwebgpu/deffered_rendering/readme.md) |<img width="500px" src="./minwebgpu/hello_triangle/showcase.jpg"><br>[Hello triangle](./minwebgpu/hello_triangle/readme.md) | |

## WGPU Examples

| | | |
|:-------------------------:|:-------------------------:|:-------------------------:|
|<img width="500px" src="./minwgpu/grid_render/showcase.png"><br>[Grid renderer](./minwgpu/grid_render/readme.md) |<img width="500px" src="./minwgpu/hello_triangle/showcase.png"><br>[Hello triangle](./minwgpu/hello_triangle/readme.md) | |
