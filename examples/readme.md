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

# Or, from any directory, by partial unique match against the example/binary path:
action/run hexagonal_grid
```

More detailed setup and run instruction: [how_to_run.md](./how_to_run.md)

## Math Examples

| | |
|:-------------------------:|:-------------------------:|
|<img width="500px" src="./math/life/showcase.webp"><br>[Game of life](./math/life/readme.md) | |

## WebGL Examples

| | |
|:-------------------------:|:-------------------------:|
|[2D line](./minwebgl/2d_line/readme.md)<br>*(No showcase yet)* |<img width="500px" src="./minwebgl/3d_line/showcase.webp"><br>[3D line](./minwebgl/3d_line/readme.md) |
|<img width="500px" src="./minwebgl/animation_amplitude_change/showcase.webp"><br>[Animation amplitude change](./minwebgl/animation_amplitude_change/readme.md) |<img width="500px" src="./minwebgl/animation_surface_rendering/showcase.webp"><br>[Animated objects surface rendering](./minwebgl/animation_surface_rendering/readme.md) |
|<img width="500px" src="./minwebgl/area_light/showcase.webp"><br>[Area light](./minwebgl/area_light/readme.md) |<img width="500px" src="./minwebgl/attributes_instanced/showcase.webp"><br>[Attributes instanced](./minwebgl/attributes_instanced/readme.md) |
|<img width="500px" src="./minwebgl/attributes_matrix/showcase.webp"><br>[Attributes matrix](./minwebgl/attributes_matrix/readme.md) |<img width="500px" src="./minwebgl/attributes_vao/showcase.webp"><br>[Attributes VAO](./minwebgl/attributes_vao/readme.md) |
|<img width="500px" src="./minwebgl/character_control/showcase.webp"><br>[Character control](./minwebgl/character_control/readme.md) |<img width="500px" src="./minwebgl/color_space_conversions/showcase.webp"><br>[Color space conversions](./minwebgl/color_space_conversions/readme.md) |
|<img width="500px" src="./minwebgl/curve_surface_rendering/showcase.webp"><br>[Curve rendering on surface](./minwebgl/curve_surface_rendering/readme.md) |<img width="500px" src="./minwebgl/deferred_shading/showcase.webp"><br>[Deferred shading](./minwebgl/deferred_shading/readme.md) |
|<img width="500px" src="./minwebgl/diamond/showcase.webp"><br>[Diamond](./minwebgl/diamond/readme.md) |<img width="500px" src="./minwebgl/filter/showcase.webp"><br>[Image filter](./minwebgl/filter/readme.md) |
|<img width="500px" src="./minwebgl/filters/showcase.webp"><br>[Image filters](./minwebgl/filters/readme.md) |<img width="500px" src="./minwebgl/gltf_viewer/showcase.webp"><br>[GLTF viewer](./minwebgl/gltf_viewer/readme.md) |
|<img width="500px" src="./minwebgl/hexagonal_grid/showcase.webp"><br>[Hexagonal grid](./minwebgl/hexagonal_grid/readme.md) |<img width="500px" src="./minwebgl/hexagonal_map/showcase.webp"><br>[Hexagonal map](./minwebgl/hexagonal_map/readme.md) |
|<img width="500px" src="./minwebgl/jewelry_site/showcase.webp"><br>[Jewelry site](./minwebgl/jewelry_site/readme.md) |<img width="500px" src="./minwebgl/lottie_surface_rendering/showcase.webp"><br>[Lottie surface rendering](./minwebgl/lottie_surface_rendering/readme.md) |
|<img width="500px" src="./minwebgl/make_cube_map/showcase.webp"><br>[Cube map](./minwebgl/make_cube_map/readme.md) |<img width="500px" src="./minwebgl/mapgen_tiles_rendering/showcase.webp"><br>[Tilemaps rendering](./minwebgl/mapgen_tiles_rendering/readme.md) |
|<img width="500px" src="./minwebgl/minimize_wasm/showcase.webp"><br>[Minimize wasm](./minwebgl/minimize_wasm/readme.md) |<img width="500px" src="./minwebgl/morph_targets/showcase.webp"><br>[Morph targets](./minwebgl/morph_targets/readme.md) |
|<img width="500px" src="./minwebgl/narrow_outline/showcase.webp"><br>[Narrow outline](./minwebgl/narrow_outline/readme.md) |<img width="500px" src="./minwebgl/obj_load/showcase.webp"><br>[OBJ loading](./minwebgl/obj_load/readme.md) |
|<img width="500px" src="./minwebgl/obj_viewer/showcase.webp"><br>[OBJ viewer](./minwebgl/obj_viewer/readme.md) |<img width="500px" src="./minwebgl/object_picking/showcase.webp"><br>[Object picking](./minwebgl/object_picking/readme.md) |
|<img width="500px" src="./minwebgl/outline/showcase.webp"><br>[Outline](./minwebgl/outline/readme.md) |<img width="500px" src="./minwebgl/pbr_lighting/showcase.webp"><br>[PBR lighting](./minwebgl/pbr_lighting/readme.md) |
|<img width="500px" src="./minwebgl/postprocessing/showcase.webp"><br>[Postprocessing](./minwebgl/postprocessing/readme.md) |<img width="500px" src="./minwebgl/raycaster/showcase.webp"><br>[Raycaster](./minwebgl/raycaster/readme.md) |
|<img width="500px" src="./minwebgl/renderer_with_outlines/showcase.webp"><br>[Outlines postprocessing](./minwebgl/renderer_with_outlines/readme.md) |[Shadowmap](./minwebgl/shadowmap/readme.md)<br>*(No showcase yet)* |
|<img width="500px" src="./minwebgl/simple_pbr/showcase.webp"><br>[Simple PBR](./minwebgl/simple_pbr/readme.md) |<img width="500px" src="./minwebgl/skeletal_animation/showcase.webp"><br>[Skeletal animation](./minwebgl/skeletal_animation/readme.md) |
|<img width="500px" src="./minwebgl/space_partition/showcase.webp"><br>[Space partition](./minwebgl/space_partition/readme.md) |<img width="500px" src="./minwebgl/spinning_cube_size_opt/showcase.webp"><br>[Spinning cube](./minwebgl/spinning_cube_size_opt/readme.md) |
|<img width="500px" src="./minwebgl/sprite_animation/showcase.webp"><br>[Sprite animation](./minwebgl/sprite_animation/readme.md) | |
|<img width="500px" src="./minwebgl/text_msdf/showcase.webp"><br>[Text MSDF](./minwebgl/text_msdf/readme.md) |<img width="500px" src="./minwebgl/text_rendering/showcase.webp"><br>[Text rendering](./minwebgl/text_rendering/readme.md) |
|[Touch input test](./minwebgl/touch_input_test/readme.md)<br>*(No showcase — manual testing aid, not a demo)* |<img width="500px" src="./minwebgl/trivial/showcase.webp"><br>[Trivial](./minwebgl/trivial/readme.md) |
|<img width="500px" src="./minwebgl/uniforms_animation/showcase.webp"><br>[Uniform animation](./minwebgl/uniforms_animation/readme.md) |<img width="500px" src="./minwebgl/uniforms_ubo/showcase.webp"><br>[Uniform UBO](./minwebgl/uniforms_ubo/readme.md) |
|<img width="500px" src="./minwebgl/video_as_texture/showcase.webp"><br>[Video as texture](./minwebgl/video_as_texture/readme.md) |<img width="500px" src="./minwebgl/wfc/showcase.webp"><br>[Wave function collapse](./minwebgl/wfc/readme.md) |

## WebGPU Examples

| | |
|:-------------------------:|:-------------------------:|
|<img width="500px" src="./minwebgpu/deffered_rendering/showcase.webp"><br>[Deffered rendering](./minwebgpu/deffered_rendering/readme.md) |<img width="500px" src="./minwebgpu/hello_triangle/showcase.webp"><br>[Hello triangle](./minwebgpu/hello_triangle/readme.md) |
|<img width="500px" src="./minwebgpu/hello_triangle_quickstart/showcase.webp"><br>[Hello triangle quickstart](./minwebgpu/hello_triangle_quickstart/readme.md) |[PBR scene (gpu_hal)](./minwebgpu/renderer_pbr_scene/readme.md)<br>*(No showcase yet)* |
|[Shader chunk preview](./minwebgpu/shader_chunk_preview/readme.md)<br>*(No showcase — see readme for why)* | |

## WGPU Examples

| | |
|:-------------------------:|:-------------------------:|
|<img width="500px" src="./minwgpu/grid_render/showcase.webp"><br>[Grid renderer](./minwgpu/grid_render/readme.md) |<img width="500px" src="./minwgpu/hello_triangle/showcase.webp"><br>[Hello triangle](./minwgpu/hello_triangle/readme.md) |

## Orrery Examples

One scene — a sun-and-orbits HUD diagram — implemented once per backend/layer; see the [family readme](./orrery/readme.md).

| | |
|:-------------------------:|:-------------------------:|
|[Orrery (WebGPU)](./orrery/webgpu/readme.md)<br>*(No showcase — see readme for why)* | |

## Scripting Examples

| | |
|:-------------------------:|:-------------------------:|
|[F32x2 vector arithmetic](./scene_script/f32x2_vector_arithmetic/readme.md)<br>*(No showcase — console output)* |[Pingpong animation](./scene_script/pingpong_animation/readme.md)<br>*(No showcase — console output)* |

## Tiles Tools Examples

| | |
|:-------------------------:|:-------------------------:|
|[Advanced pathfinding demo](./tiles_tools/advanced_pathfinding_demo/readme.md)<br>*(No showcase — console output)* |[Beginner tutorial](./tiles_tools/beginner_tutorial/readme.md)<br>*(No showcase — console output)* |
|[Debug demo](./tiles_tools/debug_demo/readme.md)<br>*(No showcase — console output)* |[ECS collision demo](./tiles_tools/ecs_collision_demo/readme.md)<br>*(No showcase — console output)* |
|[Event system demo](./tiles_tools/event_system_demo/readme.md)<br>*(No showcase — console output)* |[Field of view demo](./tiles_tools/field_of_view_demo/readme.md)<br>*(No showcase — console output)* |
|[Game of life (tiles_tools)](./tiles_tools/game_of_life/readme.md)<br>*(No showcase — console output)* |[Game systems demo](./tiles_tools/game_systems_demo/readme.md)<br>*(No showcase — console output)* |
|[Serialization demo](./tiles_tools/serialization_demo/readme.md)<br>*(No showcase — console output)* |[Simple collision demo](./tiles_tools/simple_collision_demo/readme.md)<br>*(No showcase — console output)* |
|[Stealth game](./tiles_tools/stealth_game/readme.md)<br>*(No showcase — console output)* |[Tactical RPG](./tiles_tools/tactical_rpg/readme.md)<br>*(No showcase — console output)* |

## Responsibility Table

| File | Responsibility |
|------|----------------|
| demo_completeness.md | Documentation tracking completion status of examples |
| demo_readme_example.md | Template for creating demo readme files |
| demo_todo_categorized.md | Categorized todo list for examples |
| example_requirements.md | Requirements documentation for examples |
| how_to_run.md | Setup and execution instructions for examples |
| index.html | Interactive HTML gallery with 69 example showcases |
| index.md | Markdown-formatted examples list |
| math/ | Math-based examples directory (1 demo) |
| minwebgl/ | WebGL examples directory (47 demos) |
| minwebgpu/ | WebGPU examples directory (5 demos) |
| minwgpu/ | WGPU examples directory (2 demos) |
| non_developer_how_to_run.md | Beginner-friendly step-by-step guide for running examples with no prior Rust experience |
| orrery/ | Orrery scene-family examples — one implementation per backend/layer (1 demo) |
| readme.md | Root documentation for examples directory |
| scene_script/ | Rhai scripting examples directory (2 demos) |
| tiles_tools/ | tiles_tools game-dev examples directory (12 demos) |
