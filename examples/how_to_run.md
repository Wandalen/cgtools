# How to Run CGTools Examples

This guide provides comprehensive instructions for setting up your environment and running the various demo applications in the CGTools project.

> New to Rust and setting things up for the first time? See the step-by-step [beginner guide](non_developer_how_to_run.md).

## Table of Contents

- [Prerequisites](#prerequisites)
- [Environment Setup](#environment-setup)
- [Running Examples](#running-examples)
  - [WebGL and WebGPU Examples (minwebgl / minwebgpu)](#webgl-and-webgpu-examples-minwebgl--minwebgpu)
  - [WGPU Examples (minwgpu)](#wgpu-examples-minwgpu)
  - [Math Examples](#math-examples)
  - [Scripting Examples (scene_script)](#scripting-examples-scene_script)
  - [Tiles Tools Examples (tiles_tools)](#tiles-tools-examples-tiles_tools)
- [Development Workflow](#development-workflow)
- [Troubleshooting](#troubleshooting)
- [Testing](#testing)

## Prerequisites

Before running any examples, you need to install the following tools:

### 1. Rust Toolchain

Install Rust using rustup if you haven't already:

```bash
# Install Rust (visit https://rustup.rs/ for platform-specific instructions)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Verify installation
rustc --version
cargo --version
```

### 2. WebAssembly Target

Add the WebAssembly compilation target to your Rust toolchain:

```bash
rustup target add wasm32-unknown-unknown
```

### 3. Trunk Build Tool

Install Trunk, the build tool for Rust WASM applications:

```bash
cargo install trunk
```

### 4. Optional Development Tools

For full development capabilities, install additional tools:

```bash
# Install nightly toolchain (required for some advanced testing)
rustup toolchain install nightly

# Install development tools
cargo install cargo-nextest    # Fast test runner
cargo install cargo-watch      # Auto-rebuild on file changes
cargo install cargo-wipe       # Clean build artifacts
cargo install cargo-audit      # Security vulnerability scanner
cargo install willbe           # Workspace management tool

# Install nightly-only tools
cargo +nightly install cargo-udeps  # Find unused dependencies
```

Alternatively, use the Makefile to install all tools at once:

```bash
make env-install
```

### 5. Browser Requirements

- **WebGL Examples**: Modern browser with WebGL 2.0 support (Chrome, Firefox, Edge, Safari)
- **WebGPU Examples**: Browser with WebGPU support (Chrome 113+, Edge 113+, or Firefox Nightly)
- **WGPU Examples**: These run natively, no browser required

## Running Examples

**Shortcut:** from any directory, `action/run <partial-name>` builds and runs any example or
binary below by a unique partial match against its path — e.g. `action/run trivial` or
`action/run sun_grid`. It picks the right mechanism automatically (`trunk serve --release` for
browser examples, `cargo run --release --all-features` otherwise). Run `action/run list` to see
every match candidate, or `action/run .` for its own usage. The manual per-family steps below
still apply if you want more control (dev-mode serving, watching, etc.).

### WebGL and WebGPU Examples (minwebgl / minwebgpu)

WebGL and WebGPU examples both run in the browser using WebAssembly and follow the same workflow.

Basic Example Run:

```bash
# Navigate to any example directory
cd examples/minwebgl/trivial
# or
cd examples/minwebgpu/hello_triangle

# Serve the example (development mode)
trunk serve

# Or serve in release mode (recommended for performance)
trunk serve --release

# Open browser to http://localhost:8080
```

**Note:** WebGPU examples require a browser with WebGPU support (Chrome 113+, Edge 113+, or Firefox Nightly).

### WGPU Examples (minwgpu)

WGPU examples run natively (not in browser) and output to image files.

```bash
# Navigate to WGPU example
cd examples/minwgpu/hello_triangle

# Build the example
cargo build --release

# Run the example
cargo run --release

# Output image will be saved to file
```

### Math Examples

Mathematical computation examples that run natively:

```bash
# Game of Life example
cd examples/math/life

# Run the example
cargo run --release
```

### Scripting Examples (scene_script)

Rhai scene-scripting examples that run natively and print to the console:

```bash
# Declarative pattern: a script builds an F32x2 from vector arithmetic
cd examples/scene_script/f32x2_vector_arithmetic
cargo run --release

# Imperative pattern: a script drives a ball/paddle simulation via
# callbacks, then the host tweens between two recorded frames
cd examples/scene_script/pingpong_animation
cargo run --release
```

### Tiles Tools Examples (tiles_tools)

Tile-based game-dev examples that run natively and print to the console. Each is its own crate, so no `--features` flag is needed even for the examples that use `tiles_tools`' optional ECS/serialization functionality — the required features are already set on that crate's own `tiles_tools` dependency.

```bash
cd examples/tiles_tools/advanced_pathfinding_demo && cargo run --release   # A* across obstacle/cost/multi-goal/coordinate-system variations
cd examples/tiles_tools/beginner_tutorial && cargo run --release          # step-by-step introduction to core tiles_tools concepts
cd examples/tiles_tools/debug_demo && cargo run --release                 # grid/pathfinding/ECS debug visualization and profiling
cd examples/tiles_tools/ecs_collision_demo && cargo run --release         # ECS collision detection, resolution, and spatial queries
cd examples/tiles_tools/event_system_demo && cargo run --release          # pub/sub event system with priorities and statistics
cd examples/tiles_tools/field_of_view_demo && cargo run --release         # shadowcasting, ray casting, and multi-source lighting
cd examples/tiles_tools/game_of_life && cargo run --release               # Conway's Game of Life across coordinate systems
cd examples/tiles_tools/game_systems_demo && cargo run --release          # turn-based systems, quests, and status effects
cd examples/tiles_tools/serialization_demo && cargo run --release         # save/load across JSON, binary, and RON formats
cd examples/tiles_tools/simple_collision_demo && cargo run --release      # minimal ECS collision detection walkthrough
cd examples/tiles_tools/stealth_game && cargo run --release               # field-of-view-driven stealth game with guard AI
cd examples/tiles_tools/tactical_rpg && cargo run --release               # hex-grid tactical combat with AI-controlled enemies
```

## Development Workflow

### Development Mode

Run examples with hot-reload during development:

```bash
# Navigate to example
cd examples/minwebgl/your_example

# Serve with auto-reload
trunk serve

# Make changes to src/main.rs - browser auto-refreshes
```

### Production Build

Create optimized builds for deployment:

```bash
# Build for production
trunk build --release

# Output will be in dist/ directory
# Deploy dist/ folder to web server
```

### Clean Build

Remove build artifacts:

```bash
# Clean trunk artifacts
trunk clean

# Clean Cargo build cache
cargo clean

# Or use Makefile for comprehensive clean
make cwa
```

### File Watching

Auto-run tests on file changes:

```bash
# Watch and run tests (Level 1: basic tests)
make wtest1

# Watch and run tests with linting (Level 3)
make wtest3 crate=minwebgl
```

## Troubleshooting

### Common Issues

#### Issue: "trunk: command not found"

**Solution:** Install trunk using `cargo install trunk`

#### Issue: "error: toolchain 'stable-xxx' does not support target 'wasm32-unknown-unknown'"

**Solution:** Add the WASM target:
```bash
rustup target add wasm32-unknown-unknown
```

#### Issue: Browser shows blank page

**Solutions:**
- Check browser console for errors (F12)
- Verify WebGL/WebGPU support in your browser
- Try different browser (Chrome/Firefox recommended)
- Use `trunk serve --release` for better performance
- Clear browser cache and reload

#### Issue: Examples fail to compile

**Solutions:**
```bash
# Update Rust toolchain
rustup update

# Clean and rebuild
cargo clean
trunk clean
trunk serve --release

# Check specific error messages in terminal
```

#### Issue: WebGPU examples don't work

**Solution:**
- Ensure you're using Chrome 113+ or Edge 113+
- Enable WebGPU in Firefox by visiting `about:config` and setting `dom.webgpu.enabled` to `true`
- Check if your GPU supports WebGPU

#### Issue: Performance issues

**Solutions:**
- Always use `--release` flag: `trunk serve --release`
- Check GPU acceleration is enabled in browser
- Close other GPU-intensive applications
- Update graphics drivers

## Testing

CGTools provides a leveled testing system for comprehensive quality assurance.

### Test Levels

```bash
# Level 1: Run primary test suite
make ctest1

# Level 2: Primary + Documentation tests
make ctest2

# Level 3: Primary + Doc + Linter checks
make ctest3

# Level 4: All checks + Heavy testing (deps + audit)
make ctest4

# Level 5: Full heavy testing with mutation tests
make ctest5
```

### Test Specific Crate

```bash
# Test specific example/crate
make ctest3 crate=minwebgl

# Test specific example
cd examples/minwebgl/hexagonal_grid
cargo test --all-features
```

### Full Workspace Test

Run the complete test suite as recommended in main readme:

```bash
RUSTFLAGS="-D warnings" cargo nextest run --all-features && \
RUSTDOCFLAGS="-D warnings" cargo test --doc --all-features && \
cargo clippy --all-targets --all-features -- -D warnings
```

### Browser Support

| Browser | WebGL 2.0 | WebGPU |
|---------|-----------|--------|
| Chrome | Yes | Yes (113+) |
| Firefox | Yes | Nightly only |
| Edge | Yes | Yes (113+) |
| Safari | Yes | Limited |
