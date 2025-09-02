# Examples

Interactive WebGL/WebGPU examples demonstrating CGTools capabilities.

## Quick Start

Prerequisites:
```bash
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Run example:
```bash
cd minwebgl/hexagonal_grid
trunk serve --release
# Open http://localhost:8080
```

## WebGL Examples

| Example | Description |
|---------|-------------|
| `hexagonal_grid` | Pathfinding on hex grids |
| `deferred_shading` | Multi-pass 3D rendering |
| `text_rendering` | GPU text rendering |
| `gltf_viewer` | 3D model viewer |
| `raycaster` | Ray-based rendering |
| `wave_function_collapse` | Procedural generation |

## WebGPU Examples

| Example | Description |
|---------|-------------|
| `hello_triangle` | Basic WebGPU rendering |
| `deferred_rendering` | Advanced rendering pipeline |

## Math Examples

| Example | Description |
|---------|-------------|
| `life` | Conway's Game of Life |

## Development

```bash
# Development mode
trunk serve

# Production build
trunk build --release

# Clean build
trunk clean && cargo clean
```

## Structure

```
example/
├── src/main.rs
├── Cargo.toml
├── index.html
└── assets/
```

## Troubleshooting

- Check browser console for errors
- Verify WebGL/WebGPU support
- Use `trunk serve --release` for performance testing