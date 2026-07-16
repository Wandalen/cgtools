# CGTools

Computer graphics toolkit for WebAssembly applications.

![Abstract Art](./media/primitives.jpg)

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

This repository carries demo crates (`examples/`) and the whole history. Large
binary assets live outside git (see [Assets](#assets) below), so a clone is
already smaller than it looks — but you can pull less still:

```bash
# Skip history:
git clone --depth 1 https://github.com/Wandalen/cgtools

# Fetch blobs lazily on demand instead of all up front:
git clone --filter=blob:none https://github.com/Wandalen/cgtools
```

If you only need the libraries (working under `module/`, not running the
browser demos), combine a blobless partial clone with a sparse checkout so
`examples/` is never materialized on disk:

```bash
git clone --filter=blob:none --sparse https://github.com/Wandalen/cgtools
cd cgtools
git sparse-checkout set module        # add 'examples' later if you need the demos
```

Note: these reduce what *you* download/check out; they do not change the size
of the repository on the server.

## Assets

Large binary assets — 3D models, textures, HDR environment maps, and example inputs — are **not
stored in git**. They live in a public [Hugging Face dataset](https://huggingface.co/datasets/cgtools/assets)
and are tracked with [DVC](https://dvc.org).

Cloning the repo gives you the source but not these files. The library crates (`ndarray_cg`,
`minwebgl`, …) build and test without them — you only need them to run examples that load models
or textures.

Install DVC and pull. Reads are **anonymous** — no account, credentials, or DVC extras required:

```bash
pipx install dvc          # or: uv tool install dvc  /  pip install dvc
dvc pull                  # everything (~312 MB)
```

You rarely need all of it. Browse what's tracked without downloading, then pull only the file or
subfolder you want — each file inside `assets/` is fetched individually:

```bash
dvc list -R . assets              # see the full tree first (no download)
dvc pull assets/gltf              # just the glTF models
dvc pull assets/gltf/sponza.glb   # a single file
```

Grab one file without cloning at all:

```bash
dvc get https://github.com/Wandalen/cgtools assets/gltf/sponza.glb
```

Updating assets is maintainer-only (a separate `hf upload` step); opening a PR or cloning never
pushes anything.

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

