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

```rust
use canvas_renderer::renderer::CanvasRenderer;

// Initialize the renderer
let renderer = CanvasRenderer::new(canvas_element)?;

// Render your content
renderer.render_to_canvas(texture, width, height)?;
```

### Framebuffer Rendering

```rust
// Render 3D scene to framebuffer
let framebuffer = renderer.create_framebuffer(width, height)?;
renderer.render_scene_to_framebuffer(&scene, &framebuffer)?;

// Convert framebuffer to 2D canvas
renderer.framebuffer_to_canvas(&framebuffer)?;
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
