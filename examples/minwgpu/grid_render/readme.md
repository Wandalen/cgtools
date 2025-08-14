# Hexagonal grid (WGPU render)

Example of rendering hexagonal grid with layout with WGPU.

![WGPU Triangle](showcase.png)

## 🚀 Quick Start

### Prerequisites

- Rust with native platform target

### Run the Example

``` bash
cd examples/minwgpu/grid_render
cargo run
```

## How to use

You can change path where image is saved

``` rust
image::save_buffer( "your/custom/path.png", &data, width, height, image::ColorType::Rgba8 )
```

Or colors of background, hexagons, outline

``` rust

```
