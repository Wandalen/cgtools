# CGTools

Computer graphics toolkit for WebAssembly applications.

## Overview

Rust-based graphics libraries for WebGL/WebGPU applications, mathematical computation, and game development.

## Quick Start

Prerequisites:
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Run example:
```bash
git clone https://github.com/Wandalen/cgtools
cd cgtools/examples/minwebgl/hexagonal_grid
trunk serve --release
```

## Usage

Add to `Cargo.toml`:
```toml
[dependencies]
minwebgl = "0.2"
tiles_tools = "0.1"
ndarray_cg = "0.3"
browser_input = "0.1"
```

## Core Crates

| Crate | Description |
|-------|-------------|
| `minwebgl` | WebGL 2.0 toolkit |
| `minwebgpu` | WebGPU toolkit |
| `tiles_tools` | Tile-based game systems |
| `ndarray_cg` | Computer graphics mathematics |
| `browser_input` | Input handling |
| `browser_log` | WebAssembly logging |
| `renderer` | 3D rendering system |
| `line_tools` | Line rendering |
| `embroidery_tools` | Embroidery pattern tools |

## Examples

- [Hexagonal Grid](./examples/minwebgl/hexagonal_grid/) - Interactive pathfinding
- [Deferred Shading](./examples/minwebgl/deferred_shading/) - 3D rendering pipeline
- [Text Rendering](./examples/minwebgl/text_rendering/) - GPU text rendering
- [Hello Triangle](./examples/minwebgpu/hello_triangle/) - WebGPU basics

## Development

```bash
# Test workspace
cargo test --workspace

# Run example
cd examples/minwebgl/trivial
trunk serve --release
```

## License

MIT