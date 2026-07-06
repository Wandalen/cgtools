# CGTools

Computer graphics toolkit for WebAssembly applications.

![Abstract Art](./assets/primitives.jpg)

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

### Cloning a smaller copy

This repository is large because it carries example assets (`assets/`) and demo
crates (`examples/`). A full clone downloads all of it and the whole history.
Depending on what you are doing, you can pull much less:

```bash
# Skip history (fast, keeps assets/examples so demos still run):
git clone --depth 1 https://github.com/Wandalen/cgtools

# Fetch blobs lazily on demand instead of all up front:
git clone --filter=blob:none https://github.com/Wandalen/cgtools
```

If you only need the libraries (working under `module/`, not running the
browser demos), combine a blobless partial clone with a sparse checkout so
`assets/` and `examples/` are never materialized on disk:

```bash
git clone --filter=blob:none --sparse https://github.com/Wandalen/cgtools
cd cgtools
git sparse-checkout set module        # add 'examples' 'assets' later if you need them
```

Note: these reduce what *you* download/check out; they do not change the size
of the repository on the server.

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

## Testing
Run the following command to test the entire project:
```bash
RUSTFLAGS="-D warnings" cargo nextest run --all-features && RUSTDOCFLAGS="-D warnings" cargo test --doc --all-features && cargo clippy --all-targets --all-features -- -D warnings
```

