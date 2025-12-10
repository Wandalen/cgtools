# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

CGTools is a Rust-based computer graphics toolkit for WebAssembly applications, providing libraries for WebGL/WebGPU graphics, mathematical computation, and game development. The project is organized as a Cargo workspace with modular crates.

## Building and Testing

### Prerequisites
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk  # For running examples
```

### Test Commands (Makefile Levels)
The project uses a leveled testing system via Makefile. Each level includes all previous checks:

- **Level 1 (Primary tests)**: `make ctest1`
- **Level 2 (+ Doc tests)**: `make ctest2`
- **Level 3 (+ Clippy lints)**: `make ctest3`
- **Level 4 (+ Heavy testing)**: `make ctest4` - includes cargo-udeps and cargo-audit
- **Level 5 (+ Mutation tests)**: `make ctest5`

To test a specific crate: `make ctest3 crate=minwebgl`

### Standard Test Command
```bash
RUSTFLAGS="-D warnings" cargo nextest run --all-features && RUSTDOCFLAGS="-D warnings" cargo test --doc --all-features && cargo clippy --all-targets --all-features -- -D warnings
```

Note: The project uses `cargo nextest` for running tests, not `cargo test`.

### Running Examples
Examples are WebAssembly applications served via Trunk:
```bash
cd examples/minwebgl/hexagonal_grid
trunk serve --release
```

### Watch Mode
Use watch commands for continuous testing during development:
- `make wtest1` - Watch primary tests
- `make wtest3` - Watch with linting (recommended for active development)

## Architecture

### Workspace Structure

The workspace is organized into several categories:

**Core Graphics Libraries (`module/min/`)**
- `minwebgl` - WebGL 2.0 toolkit with shader management, buffers, attributes, textures, and framebuffers
- `minwebgpu` - WebGPU toolkit for modern GPU APIs
- `mingl` - Shared graphics abstractions and utilities

**Math Libraries (`module/math/`)**
- `ndarray_cg` - Computer graphics mathematics based on ndarray
  - Organized by dimensionality: `d2/`, `quaternion/`, `vector/`
  - Matrix types: `mat2x2`, `mat3x3`, `mat4x4` with both column and row-major variants (suffix 'h')
  - Custom lints allow mathematical naming conventions (single-char variables, many arguments)
- `mdmath_core` - Core mathematical operations and traits

**Helper Libraries (`module/helper/`)**
- `renderer` - 3D rendering system with glTF loading and scene management
- `tiles_tools` - Tile-based game systems
- `browser_input` - Input handling for web applications
- `browser_log` - WebAssembly logging utilities
- `animation` - Animation system support
- `line_tools` - Line rendering utilities
- `primitive_generation` - Mesh generation utilities
- `tilemap_renderer` - Tile map rendering
- `behaviour_tree` - AI behavior trees

**Alias Modules (`module/alias/`)**
- `browser_tools` - Aggregates browser-related utilities
- `ndarray_tools` - Aggregates ndarray utilities

**Blank Modules (`module/blank/`)**
- Placeholder/namespace crates for future expansion

### Examples Organization

Examples are categorized by rendering backend:
- `examples/minwebgl/*` - WebGL examples (largest collection)
- `examples/minwebgpu/*` - WebGPU examples
- `examples/minwgpu/*` - Native WGPU examples
- `examples/math/*` - Math library examples

Key example categories:
- **Rendering techniques**: deferred_shading, shadowmap, simple_pbr, area_light
- **Asset loading**: gltf_viewer, obj_viewer, obj_load
- **Text**: text_rendering, text_msdf
- **Animation**: skeletal_animation, sprite_animation, animation_surface_rendering
- **Interactive**: hexagonal_grid, hexagonal_map, object_picking, raycaster
- **Optimization**: minimize_wasm, spinning_cube_size_opt

## Key Technical Details

### WebAssembly Configuration
The project uses unstable web_sys APIs for WebGPU support. Configuration is in `.cargo/config.toml`:
```toml
[build]
rustflags = ["--cfg", "web_sys_unstable_apis"]
```

### Linting Philosophy
The workspace enforces strict linting (workspace.lints in Cargo.toml):
- `rust_2018_idioms = "deny"`
- `missing_docs = "warn"`
- `unsafe-code = "warn"`
- Clippy pedantic mode with specific allows for mathematical code
- `undocumented_unsafe_blocks = "deny"`

Math crates like `ndarray_cg` have relaxed lints for mathematical idioms (single-char names, many arguments).

### Feature System
Most crates use an "enabled" feature that gates all functionality:
```toml
[features]
enabled = ["dep:foo", "dep:bar"]
default = ["enabled"]
```

This pattern allows conditional compilation and reduces build times.

### Examples Structure
Each example typically contains:
- `Cargo.toml` - Dependencies (usually minwebgl/renderer)
- `index.html` - Basic HTML page (Trunk injects the WASM)
- `src/main.rs` - Entry point with `#[wasm_bindgen(start)]`
- Optional `dist/` or `assets/` directories for static resources

### Matrix Memory Layout
`ndarray_cg` provides both column-major (default) and row-major (suffix 'h') matrix types:
- `Mat4x4` - Column-major (OpenGL convention)
- `Mat4x4h` - Row-major (alternative layout)

This dual approach accommodates different shader conventions.

## Development Workflow

### Adding a New Example
1. Create directory under appropriate backend: `examples/minwebgl/my_example/`
2. Add `Cargo.toml` with necessary dependencies
3. Create `index.html` (can copy from trivial example)
4. Implement `src/main.rs` with `#[wasm_bindgen(start)]` entry point
5. Test with `trunk serve --release`
6. Add to workspace members in root `Cargo.toml` if needed

### Working with Shaders
Shaders in minwebgl are typically:
- GLSL ES 3.00 for WebGL 2.0
- Included as string literals in Rust source
- Compiled at runtime via `minwebgl::shader::make()`

### Branch Naming Convention
From conventions.md:
- Create branches prefixed with your GitHub name: `<Github name>/<branch name>`
- Or fork and create PRs from your fork

## Common Pitfalls

1. **WASM target not installed**: Must run `rustup target add wasm32-unknown-unknown`
2. **Using cargo test instead of nextest**: The project standardizes on `cargo nextest`
3. **Missing trunk**: Examples require trunk to be installed
4. **Forgetting --release**: Examples run much faster with `trunk serve --release`
5. **Web-sys unstable APIs**: Some features require the web_sys_unstable_apis cfg (already configured)

## Module Documentation Standards

From conventions.md, each module should have:
- Description and purpose
- Documentation of caveats and non-obvious technical decisions
- Relevant links to resources

Shader files should document:
- Purpose of the shader
- Any caveats or limitations
- Relevant technical references
