# 🔬 WebGL Examples with `minwebgl`

Welcome to the `minwebgl` examples directory! This collection showcases a wide variety of WebGL rendering techniques, from fundamental concepts to advanced graphics pipelines, all implemented in Rust and compiled to WebAssembly.

Each example is a self-contained project designed to demonstrate a specific feature, algorithm, or use case, making this a valuable resource for learning modern, high-performance web graphics with Rust.

## 🚀 How to Run the Examples

All examples are built using Rust and the `trunk` build tool to simplify the WebAssembly development workflow.

### Prerequisites

1. **Rust**: Ensure you have a recent version of the Rust toolchain installed. You can get it from [rustup.rs](https://rustup.rs/).
2. **Wasm Target**: Add the WebAssembly target to your Rust toolchain:

    ```bash
    rustup target add wasm32-unknown-unknown
    ```

3. **Trunk**: Install the `trunk` build tool, which handles Wasm compilation, asset management, and serving.

    ```bash
    cargo install trunk
    ```

### Running a Specific Example

To run any of the examples, navigate to its directory and use the `trunk serve` command. For best performance, always use `--release` mode.

```bash
# 1. Navigate to the desired example's directory
cd examples/minwebgl/<example_name>

# 2. Build and serve the application in release mode
trunk serve --release

# 3. Open your browser to the provided address (usually http://127.0.0.1:8080)
```

For example, to run the `gltf_viewer` demo:

```bash
cd examples/minwebgl/gltf_viewer
trunk serve --release
```

## 📂 Table of Contents

Below is a list of all available examples, each demonstrating a unique WebGL concept.

| Example | Description |
| :--- | :--- |
| **Rendering Fundamentals** | |
| `trivial` | A minimal "hello world" that draws a single point on the screen. |
| `attributes_vao` | Demonstrates using a Vertex Array Object (VAO) to manage multiple vertex attributes. |
| `attributes_instanced` | Shows how to use instanced drawing to render many objects with a single draw call. |
| `attributes_matrix` | An advanced instancing example that passes transformation matrices as per-instance attributes. |
| `uniforms_animation` | A basic example of animating an object by updating shader uniforms every frame. |
| `uniforms_ubo` | Shows the use of Uniform Buffer Objects (UBOs) for efficient management of shared uniform data. |
| **2D Graphics & Rendering** | |
| `2d_line` | Renders dynamic 2D lines with customizable joins and caps using procedural geometry. |
| `sprite_animation` | Implements an efficient sprite animation system using GPU texture arrays. |
| `raycaster` | A classic 2.5D raycasting engine that renders a 2D map into a pseudo-3D perspective, like in Wolfenstein 3D. |
| `hexagonal_grid` | An interactive hexagonal grid system featuring coordinate conversion and A* pathfinding. |
| `hexagonal_map` | A full-featured hexagonal map editor with tile painting, rivers, and save/load functionality. |
| `mapgen_tiles_rendering` | An efficient tilemap rendering system using texture arrays and unsigned integer textures. |
| `wfc` | Procedurally generates tilemaps from a sample pattern using the Wave Function Collapse algorithm. |
| **Text Rendering** | |
| `text_msdf` | Renders high-quality, infinitely scalable text using Multi-channel Signed Distance Fields (MSDF). |
| `text_rendering` | Generates and renders true 3D text geometry from TTF and UFO font files. |
| **3D Graphics & Models** | |
| `obj_load` | A basic example of loading and rendering a 3D model from a `.obj` file. |
| `obj_viewer` | A more advanced OBJ model viewer with full MTL material and texture support. |
| `gltf_viewer` | A production-quality PBR viewer for glTF 2.0 models with IBL and post-processing. |
| `3d_line` | Renders dynamic, animates 3D lines in a scene with interactive orbit controls. |
| `spinning_cube_size_opt` | A spinning cube demo that showcases effective techniques for reducing WASM bundle size. |
| **Advanced Rendering & PBR** | |
| `simple_pbr` | A minimal, shader-only example demonstrating the core principles of Physically-Based Rendering. |
| `area_light` | A PBR showcase with realistic, real-time rectangular area lights using Linearly Transformed Cosines (LTCs). |
| `deferred_shading` | A high-performance deferred shading pipeline that supports hundreds of dynamic lights using light volumes. |
| `diamond` | A specialized renderer for a diamond, simulating complex refraction and dispersion effects using cube maps. |
| **Post-Processing & Effects** | |
| `filter` | A simple example applying a single, interactive emboss filter to an image. |
| `filters` | A comprehensive demo of various GPU-accelerated image filters applied in real-time. |
| `outline` | Implements a real-time object outline using the multi-pass Jump Flooding Algorithm (JFA). |
| `narrow_outline` | A post-processing example for rendering a crisp, narrow outline around 3D objects. |
| **Textures & Surfaces** | |
| `video_as_texture` | Shows how to use an HTML5 video element as a dynamic, animated texture in a 3D scene. |
| `make_cube_map` | Demonstrates the dynamic generation of cube maps by rendering a scene from six camera angles. |
| `curve_surface_rendering` | Renders 2D curves and text generated from font files directly onto a 3D surface. |
| `animation_surface_rendering` | Renders a complex 2D vector animation onto the surface of a 3D sphere. |
| **Interaction & Tools** | |
| `object_picking` | Demonstrates precise, GPU-based object selection in a 3D scene using a color-coded ID buffer. |
| `color_space_conversions` | An interactive tool to visualize real-time conversions between numerous color spaces. |
| **Optimization** | |
| `minimize_wasm` | A guide and example for optimizing the size of WebAssembly builds using `wee_alloc` and `wasm-opt`. |
