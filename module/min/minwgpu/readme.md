# minwgpu

Minimal, opinionated `wgpu` toolkit for native (off-browser) graphics.

## Overview

`minwgpu` reduces the boilerplate of common `wgpu` patterns — setting up a
context, creating buffers and textures, building render/compute pipelines —
behind small, convenient builders and helpers. It's the native counterpart to
[`minwebgl`](../minwebgl/) and [`minwebgpu`](../minwebgpu/): the same
"minimal, type-safe wrapper" philosophy, targeting native `wgpu` (driven via
`pollster` for blocking) instead of the browser.

## Modules

| Module | Responsibility |
|--------|-----------------|
| `context` | `wgpu` instance/adapter/device/queue setup |
| `helper` | Adapter and device selection helpers |
| `buffer` | Buffer creation and upload |
| `texture` | Texture creation and views |
| `surface` | Window surface format selection and configuration |
| `bind` | Bind group and bind group layout builders |
| `pipeline` | Render and compute pipeline builders |
| `pass` | Render and compute pass helpers |
| `readback` | GPU → CPU buffer/texture readback |
| `error` | Crate error type |

## Examples

- [`hello_triangle`](../../../examples/minwgpu/hello_triangle/) — minimal triangle render
- [`grid_render`](../../../examples/minwgpu/grid_render/) — instanced grid rendering

## Testing

```bash
cargo test -p minwgpu
```

## Documentation

Design documentation (features) lives in [`docs/`](docs/feature/readme.md).

## Directory Layout

| Path | Responsibility |
|------|----------------|
| `src/` | Crate source — context, buffer, texture, pipeline, and pass wrappers |
| `docs/` | Design documentation as typed doc definitions — see [docs/feature/readme.md](docs/feature/readme.md) |
| `tests/` | Integration tests |
| `readme.md` | This file — user-facing entry point |
