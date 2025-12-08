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
[Animated Objects Surface Rendering](./minwebgl/animation_surface_rendering/readme.md) | Surface rendering with animated objects and easing functions
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
[Raycaster](./minwebgl/raycaster/readme.md) | Raycasting implementation with controls
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
[Hexagonal Grid](./minwebgl/hexagonal_grid/readme.md) | Hexagonal grid system implementation
[Hexagonal Map](./minwebgl/hexagonal_map/readme.md) | Complete hexagonal map with triaxial coordinates
[Tilemaps Rendering](./minwebgl/mapgen_tiles_rendering/readme.md) | Tile-based map rendering
[Wave Function Collapse](./minwebgl/wfc/readme.md) | Procedural generation using wave function collapse algorithm

### Optimization

Example | Description
--- | ---
[Attributes Instanced](./minwebgl/attributes_instanced/readme.md) | Instanced rendering for efficient drawing
[Attributes Matrix](./minwebgl/attributes_matrix/readme.md) | Matrix attribute handling
[Attributes VAO](./minwebgl/attributes_vao/readme.md) | Vertex Array Object (VAO) usage
[Uniform UBO](./minwebgl/uniforms_ubo/readme.md) | Uniform Buffer Objects for efficient uniform handling
[Minimize WASM](./minwebgl/minimize_wasm/readme.md) | WASM size optimization techniques
[Derive Tools Issue](./minwebgl/derive_tools_issue/readme.md) | Debugging and optimization example

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
