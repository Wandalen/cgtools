# canvas_renderer

2D canvas renderer for WebGL applications with framebuffer rendering and 3D scene support.

[![Crates.io](https://img.shields.io/crates/v/canvas_renderer.svg)](https://crates.io/crates/canvas_renderer)
[![Documentation](https://docs.rs/canvas_renderer/badge.svg)](https://docs.rs/canvas_renderer)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Canvas Rendering**: High-performance 2D canvas rendering for WebGL applications
- **Framebuffer Support**: Render to texture capabilities with WebGL framebuffers
- **3D Scene Integration**: Support for rendering 3D scenes to 2D canvas output
- **WebAssembly Optimized**: Designed specifically for WASM environments
- **Modular Design**: Clean API with optional feature flags

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
canvas_renderer = "0.1.0"
```

For full functionality, enable the `full` feature:

```toml
[dependencies]
canvas_renderer = { version = "0.1.0", features = ["full"] }
```

## Features

- `enabled` (default): Core canvas rendering functionality
- `full`: All features enabled

## Usage

### Basic Canvas Rendering

```rust,no_run
# use canvas_renderer::renderer::CanvasRenderer;
# use minwebgl as gl;
# use std::error::Error;
# 
# fn example() -> Result<(), Box<dyn Error>> {
# let gl_context: &gl::GL = todo!("Get WebGL context from canvas");
# let width = 800u32;
# let height = 600u32;

// Initialize the renderer
let renderer = CanvasRenderer::new(gl_context, width, height)?;

// Access the output texture
let _output_texture = renderer.get_texture();

# Ok(())
# }
```

### Framebuffer Rendering

```rust,no_run
# use canvas_renderer::renderer::CanvasRenderer;
# use minwebgl as gl;
# use renderer::webgl::{Scene, Camera};
# use minwebgl::F32x4;
# use std::error::Error;
# 
# fn example() -> Result<(), Box<dyn Error>> {
# let gl_context: &gl::GL = todo!("Get WebGL context from canvas");
# let width = 800u32;
# let height = 600u32;
# let mut scene: Scene = todo!("Create scene");
# let camera: Camera = todo!("Create camera");
# let colors = vec![F32x4::from_array([1.0, 0.0, 0.0, 1.0])];

let renderer = CanvasRenderer::new(gl_context, width, height)?;

// Render 3D scene to internal framebuffer
renderer.render(gl_context, &mut scene, &camera, &colors)?;

// Access the rendered texture
let _output_texture = renderer.get_texture();

# Ok(())
# }
```

## Platform Support

This crate is designed for WebAssembly environments and supports:

- `wasm32-unknown-unknown` (primary target)
- Native builds for development and testing

## Dependencies

- `minwebgl`: WebGL context management and utilities
- `mingl`: Mathematics and 3D graphics utilities
- `renderer`: Core rendering functionality
- `web-sys`: Browser API bindings

## License

Licensed under the MIT License. See [LICENSE](license) file for details.

## Contributing

Contributions are welcome! Please see the [repository](https://github.com/Wandalen/cgtools) for contribution guidelines.
