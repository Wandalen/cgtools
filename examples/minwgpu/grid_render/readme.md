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

You can change path where image is saved:

``` rust
image::save_buffer( "your/custom/path.png", &data, width, height, image::ColorType::Rgba8 )
```

Or colors of background, hexagons, outline:

``` rust
let clear_color = wgpu::Color
{
  r : 0.5,
  g : 0.5,
  b : 0.5,
  a : 1.0,
};
let hexagon_color = [ 0.0_f32, 0.0, 0.0 ];
let outline_color = [ 1.0_f32, 1.0, 1.0 ];
```
