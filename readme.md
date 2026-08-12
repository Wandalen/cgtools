# CGTools

[![CI](https://github.com/Wandalen/cgtools/actions/workflows/ci.yml/badge.svg)](https://github.com/Wandalen/cgtools/actions/workflows/ci.yml)

Computer graphics toolkit for WebAssembly applications.

![Abstract Art](./assets/media/primitives.jpg)

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

Browse the full [example gallery](./examples/index.md) for every example across WebGL, WebGPU, WGPU, and Rhai scripting.

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

# Or, from any directory, by partial unique match against the example/binary path:
action/run trivial
```

## Testing

```bash
# Full verification: native suite (nextest + doctests + clippy), plus a wasm32
# compile check across every browser-kind example and the actual
# wasm_bindgen_test suites (browser-driven).
verb/test

# Ordinary, scoped verification during development (single package).
verb/test_only pkg::<crate>
```

Never prefix these with `RUSTFLAGS`/`RUSTDOCFLAGS` env vars — `.cargo/config.toml`
already sets the `--cfg` flags this workspace needs to compile at all (e.g.
`web_sys_unstable_apis`); an env var replaces those wholesale instead of merging.

## Installing workspace binaries

```bash
# Install every bin-target crate under module/ (examples/ demos are excluded).
verb/install/run

# Preview the crate set without installing; or install one crate by name.
verb/install/run dry::1
verb/install/run shader_chunks
```

