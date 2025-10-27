# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a WebGL 2.0 image processing demo showcasing 25+ GPU-accelerated filters running in real-time. The project is a Rust WebAssembly application that demonstrates post-processing techniques from basic convolution kernels to advanced multi-pass rendering algorithms.

**Location in Repository**: `examples/minwebgl/filters/`

## Build & Run Commands

### Development
```bash
# Serve with auto-reload (development mode)
trunk serve

# Serve with optimizations (recommended for testing performance)
trunk serve --release
```

### Production
```bash
# Build for deployment
trunk build --release

# Output will be in dist/ directory
```

### Cleanup
```bash
# Clean trunk artifacts
trunk clean

# Clean Cargo build cache
cargo clean
```

### Testing
```bash
# Run tests for this crate
cargo test --all-features

# Run workspace tests from repository root
cd ../../..
RUSTFLAGS="-D warnings" cargo nextest run --all-features

# Run with doc tests and linter
make ctest3 crate=filters
```

## Architecture

### Core Components

1. **Filter Trait System** (`src/filters/mod.rs`)
   - `Filter` trait: Defines the interface all filters must implement
     - `glsl_fragment_source()`: Returns GLSL fragment shader code
     - `draw()`: Executes the rendering logic
   - `FilterRenderer` trait: Provides renderer capabilities to filters
     - Access to GL context, textures, programs, and framebuffers
   - 25+ filter implementations in separate modules (blur, sharpen, twirl, etc.)

2. **Renderer** (`src/renderer.rs`)
   - Central rendering engine that applies filters
   - Manages WebGL program compilation and caching
   - Only recompiles shaders when filter source changes
   - Handles framebuffer management for multi-pass effects
   - Uses a shared vertex shader (`src/shaders/main.vert`)

3. **Framebuffer System** (`src/framebuffer.rs`)
   - Manages off-screen render targets for multi-pass effects
   - Creates color attachments with proper texture parameters
   - Required for filters that need intermediate rendering steps (e.g., blur)

4. **UI System** (`src/ui_setup.rs`)
   - Dynamic filter selection via sidebar buttons
   - Parameter controls using lil-gui library
   - Two types of filters:
     - Simple filters: No parameters (original, grayscale, invert, etc.)
     - Parametric filters: Adjustable via sliders/dropdowns (blur size, gamma value, etc.)
   - Uses serde for serializing/deserializing filter state from JavaScript

5. **JavaScript Bridge** (`gui.js`, `src/lil_gui.rs`)
   - Wraps lil-gui library for parameter controls
   - Rust exports functions that JavaScript imports
   - Bidirectional communication using wasm-bindgen

### Rendering Flow

1. Image loads → WebGL texture created
2. User selects filter → Renderer compiles shader if needed
3. Filter.draw() called → Applies effect to image
4. Multi-pass filters (blur, etc.) use framebuffer as intermediate target
5. Final result rendered to canvas

### Multi-Pass Rendering

Some filters (e.g., blur) require two passes:
1. **Pass 1**: Render to framebuffer with horizontal direction
2. **Pass 2**: Render to canvas with vertical direction using framebuffer texture

This separable filtering approach is more efficient than a single 2D pass.

## Adding New Filters

To add a new filter:

1. Create a new module in `src/filters/` (e.g., `my_filter.rs`)
2. Implement the `Filter` trait:
   ```rust
   pub struct MyFilter { pub strength: f32 }

   impl Filter for MyFilter {
       fn glsl_fragment_source(&self) -> String {
           // Return GLSL 3.00 ES shader code
       }

       fn draw(&self, renderer: &impl FilterRenderer) {
           // Upload uniforms and call default_render_pass or custom logic
       }
   }
   ```
3. Add `pub mod my_filter;` to `src/filters/mod.rs`
4. Register in UI (`src/ui_setup.rs`):
   - Add button to `generate_filter_buttons()`
   - Add to `setup_filters_without_gui()` (simple) or `setup_filters_with_gui()` (parametric)
5. Make struct serializable with `#[derive(Serialize, Deserialize)]` if using GUI controls

## Code Conventions

- GLSL version: `#version 300 es` (WebGL 2.0)
- Shaders are embedded as strings in Rust code
- Fragment shader inputs: `v_tex_coord` (vec2)
- Fragment shader outputs: `frag_color` (vec4)
- Texture uniform: `u_image` (sampler2D)
- The project uses extensive clippy allow directives at the top of main.rs

## Dependencies

- `minwebgl`: Workspace crate providing WebGL 2.0 bindings and utilities
- `web-sys`: Low-level WebAssembly browser APIs
- `serde` + `serde-wasm-bindgen`: JavaScript interop for filter parameters
- External: `lil-gui` (via CDN in gui.js)

## Key Files

- `src/main.rs`: Entry point, image loading, GL context setup
- `src/renderer.rs`: Core rendering engine
- `src/filters/mod.rs`: Filter trait definitions
- `src/filters/*.rs`: Individual filter implementations
- `src/framebuffer.rs`: Off-screen rendering targets
- `src/ui_setup.rs`: UI generation and event handling
- `src/lil_gui.rs`: JavaScript GUI library bindings
- `index.html`: HTML template with canvas and sidebar
- `gui.js`: JavaScript module exporting lil-gui wrapper functions
- `Cargo.toml`: Package configuration and dependencies

## WebGL-Specific Notes

- Images are flipped on Y-axis during upload (`UNPACK_FLIP_Y_WEBGL`)
- Texture wrapping set to `CLAMP_TO_EDGE` to avoid artifacts at borders
- Mipmaps generated for better quality when downscaling
- Drawing uses a fullscreen triangle (3 vertices, no buffers needed - geometry generated in vertex shader)
- Viewport matches canvas/framebuffer dimensions

## Repository Context

This is one example in the larger cgtools repository:
- Main repo focuses on computer graphics tools for WebAssembly
- Sister examples demonstrate other techniques (3D rendering, PBR, deferred shading, etc.)
- Shared workspace with common crates like `minwebgl`, `ndarray_cg`, `browser_input`
