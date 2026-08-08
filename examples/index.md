# Examples

Interactive WebGL/WebGPU examples demonstrating CGTools capabilities.

## Table of Contents

- [Examples](#examples)
  - [Table of Contents](#table-of-contents)
  - [Quick Start](#quick-start)
  - [Math Examples](#math-examples)
  - [WebGL Examples](#webgl-examples)
    - [2D Rendering](#2d-rendering)
    - [3D Rendering](#3d-rendering)
    - [Animation](#animation)
    - [Graphics Techniques](#graphics-techniques)
    - [Image Processing](#image-processing)
    - [Text Rendering](#text-rendering)
    - [Asset Loading](#asset-loading)
    - [Game Development](#game-development)
    - [Optimization](#optimization)
  - [WebGPU Examples](#webgpu-examples)
  - [WGPU Examples](#wgpu-examples)
  - [Scripting Examples](#scripting-examples)
  - [Tiles Tools Examples](#tiles-tools-examples)
  - [Development](#development)
  - [Structure](#structure)
  - [Troubleshooting](#troubleshooting)

## Math Examples

Example | Description
--- | ---
[Game of Life](./math/life/readme.md) | Conway's Game of Life cellular automaton simulation

## WebGL Examples

### 2D Rendering

Example | Description
--- | ---
[2D Line](./minwebgl/2d_line/readme.md) | Demonstrates 2D line rendering with interactive controls
[Diamond](./minwebgl/diamond/readme.md) | Diamond shape rendering demonstration
[Sprite Animation](./minwebgl/sprite_animation/readme.md) | Sprite-based animation system

### 3D Rendering

Example | Description
--- | ---
[3D Line](./minwebgl/3d_line/readme.md) | 3D line rendering with simulation
[Simple PBR](./minwebgl/simple_pbr/readme.md) | Simple physically-based rendering implementation
[Spinning Cube](./minwebgl/spinning_cube_size_opt/readme.md) | Optimized spinning cube demonstration
[Trivial](./minwebgl/trivial/readme.md) | Minimal WebGL rendering example

### Animation

Example | Description
--- | ---
[Animation Amplitude Change](./minwebgl/animation_amplitude_change/readme.md) | Animation with dynamic amplitude changes
[Animated Objects Surface Rendering](./minwebgl/animation_surface_rendering/readme.md) | Surface rendering with animated objects and easing functions
[Lottie Surface Rendering](./minwebgl/lottie_surface_rendering/readme.md) | Renders Lottie animations on 3D surfaces with proper hierarchy and transforms
[Morph Targets](./minwebgl/morph_targets/readme.md) | Mesh morphing and blending demonstration
[Skeletal Animation](./minwebgl/skeletal_animation/readme.md) | Skeletal/bone-based character animation
[Uniform Animation](./minwebgl/uniforms_animation/readme.md) | Animation using shader uniforms

### Graphics Techniques

Example | Description
--- | ---
[Area Light](./minwebgl/area_light/readme.md) | Area lighting implementation
[Deferred Shading](./minwebgl/deferred_shading/readme.md) | Deferred shading rendering pipeline
[Curve Surface Rendering](./minwebgl/curve_surface_rendering/readme.md) | Rendering curves on 3D surfaces
[Narrow Outline](./minwebgl/narrow_outline/readme.md) | Narrow outline effect rendering
[Outline](./minwebgl/outline/readme.md) | Object outline rendering
[Outlines Postprocessing](./minwebgl/renderer_with_outlines/readme.md) | Renderer with outline postprocessing effects
[Object Picking](./minwebgl/object_picking/readme.md) | Interactive object picking/selection
[PBR Lighting](./minwebgl/pbr_lighting/readme.md) | Advanced physically-based rendering lighting techniques
[Postprocessing](./minwebgl/postprocessing/readme.md) | Postprocessing effects pipeline
[Raycaster](./minwebgl/raycaster/readme.md) | Raycasting implementation with controls
[Shadowmap](./minwebgl/shadowmap/readme.md) | Shadow mapping implementation
[Video as Texture](./minwebgl/video_as_texture/readme.md) | Using video as texture source

### Image Processing

Example | Description
--- | ---
[Image Filter](./minwebgl/filter/readme.md) | Single image filter demonstration
[Image Filters](./minwebgl/filters/readme.md) | Comprehensive collection of image filters including blur, sharpen, edge detection, color adjustments, and artistic effects
[Color Space Conversions](./minwebgl/color_space_conversions/readme.md) | Color space conversion utilities

### Text Rendering

Example | Description
--- | ---
[Text MSDF](./minwebgl/text_msdf/readme.md) | Multi-channel signed distance field text rendering
[Text Rendering](./minwebgl/text_rendering/readme.md) | Basic text rendering techniques

### Asset Loading

Example | Description
--- | ---
[GLTF Viewer](./minwebgl/gltf_viewer/readme.md) | GLTF model viewer with IBL and HDR support
[OBJ Loading](./minwebgl/obj_load/readme.md) | OBJ file format loading
[OBJ Viewer](./minwebgl/obj_viewer/readme.md) | Complete OBJ model viewer with materials
[Cube Map](./minwebgl/make_cube_map/readme.md) | Cube map generation and usage

### Game Development

Example | Description
--- | ---
[Character Control](./minwebgl/character_control/readme.md) | Character movement and control system
[Hexagonal Grid](./minwebgl/hexagonal_grid/readme.md) | Hexagonal grid system implementation
[Hexagonal Map](./minwebgl/hexagonal_map/readme.md) | Complete hexagonal map with triaxial coordinates
[Tilemaps Rendering](./minwebgl/mapgen_tiles_rendering/readme.md) | Tile-based map rendering
[Wave Function Collapse](./minwebgl/wfc/readme.md) | Procedural generation using wave function collapse algorithm
[Touch Input Test](./minwebgl/touch_input_test/readme.md) | Manual testing aid for touch, swipe, and pinch-zoom input on mobile

### Optimization

Example | Description
--- | ---
[Attributes Instanced](./minwebgl/attributes_instanced/readme.md) | Instanced rendering for efficient drawing
[Attributes Matrix](./minwebgl/attributes_matrix/readme.md) | Matrix attribute handling
[Attributes VAO](./minwebgl/attributes_vao/readme.md) | Vertex Array Object (VAO) usage
[Uniform UBO](./minwebgl/uniforms_ubo/readme.md) | Uniform Buffer Objects for efficient uniform handling
[Minimize WASM](./minwebgl/minimize_wasm/readme.md) | WASM size optimization techniques
[Derive Tools Issue](./minwebgl/derive_tools_issue/readme.md) | Debugging and optimization example
[Space Partition](./minwebgl/space_partition/readme.md) | Spatial partitioning data structures for efficient collision detection

## WebGPU Examples

Example | Description
--- | ---
[Deferred Rendering](./minwebgpu/deffered_rendering/readme.md) | Deferred rendering pipeline using WebGPU
[Hello Triangle](./minwebgpu/hello_triangle/readme.md) | Basic WebGPU triangle rendering

## WGPU Examples

Example | Description
--- | ---
[Grid Renderer](./minwgpu/grid_render/readme.md) | Grid rendering using WGPU
[Hello Triangle](./minwgpu/hello_triangle/readme.md) | Basic WGPU triangle rendering

## Scripting Examples

Example | Description
--- | ---
[F32x2 Vector Arithmetic](./scene_script/f32x2_vector_arithmetic/readme.md) | Rhai script building an F32x2 value from vector arithmetic
[Pingpong Animation](./scene_script/pingpong_animation/readme.md) | Rhai-driven ball/paddle simulation tweened with animation::Tween

## Tiles Tools Examples

Example | Description
--- | ---
[Advanced Pathfinding Demo](./tiles_tools/advanced_pathfinding_demo/readme.md) | A* pathfinding across obstacles, costs, multi-goal search, and coordinate systems
[Beginner Tutorial](./tiles_tools/beginner_tutorial/readme.md) | Step-by-step introduction to tiles_tools' core concepts
[Debug Demo](./tiles_tools/debug_demo/readme.md) | Grid, pathfinding, and ECS debug visualization and profiling tools
[ECS Collision Demo](./tiles_tools/ecs_collision_demo/readme.md) | ECS collision detection, resolution, and spatial queries
[Event System Demo](./tiles_tools/event_system_demo/readme.md) | Decoupled pub/sub event system with priorities and statistics
[Field of View Demo](./tiles_tools/field_of_view_demo/readme.md) | Shadowcasting, ray casting, and multi-source lighting algorithms
[Game of Life (tiles_tools)](./tiles_tools/game_of_life/readme.md) | Conway's Game of Life via tiles_tools ECS across coordinate systems
[Game Systems Demo](./tiles_tools/game_systems_demo/readme.md) | Turn-based systems, resource management, quests, and status effects
[Serialization Demo](./tiles_tools/serialization_demo/readme.md) | Save/load functionality across JSON, binary, and RON formats
[Simple Collision Demo](./tiles_tools/simple_collision_demo/readme.md) | Minimal ECS collision detection walkthrough
[Stealth Game](./tiles_tools/stealth_game/readme.md) | Field-of-view-driven stealth game with guard AI
[Tactical RPG](./tiles_tools/tactical_rpg/readme.md) | Hex-grid tactical combat with AI-controlled enemies
