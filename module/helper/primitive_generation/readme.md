# primitive_generation

3D geometry generation toolkit with primitives, text rendering, and procedural shape creation.

[![Crates.io](https://img.shields.io/crates/v/primitive_generation.svg)](https://crates.io/crates/primitive_generation)
[![Documentation](https://docs.rs/primitive_generation/badge.svg)](https://docs.rs/primitive_generation)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **Primitive Data Model**: `PrimitiveData`/`AttributesData`/`Transform` plus `primitives_data_to_gltf` to assemble primitives into a renderable `renderer` GLTF scene
- **Curve Meshing**: `curve_to_geometry` turns a 2D polyline into a triangulated ribbon of given width; `plane_to_geometry` for full-screen quads
- **Path Flattening** (`text`): `path_to_points` flattens a `kurbo` path (curves included) into a point sequence
- **Text Rendering** (`font-processing`): load UFO fonts and convert strings to triangulated 3D meshes — `load_fonts`, `text_to_mesh`, `text_to_countour_mesh`, `contours_to_fill_geometry`
- **WebAssembly Ready**: font loading fetches over the network; used by the `text_rendering`, `curve_surface_rendering`, and `animation_surface_rendering` examples

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
primitive_generation = "0.1.0"
```

For full functionality, enable all features:

```toml
[dependencies]
primitive_generation = { version = "0.1.0", features = ["full"] }
```

## Feature Flags

- `enabled` (default): primitive data model, curve meshing, GLTF assembly
- `text`: `kurbo` path flattening (`path_to_points`)
- `font-processing`: UFO font loading and text-to-mesh (implies `text`; adds `earcutr`, `norad`, `quick-xml`)
- `full`: everything above

Verify what a flag actually exports: `cargo doc -p primitive_generation --features font-processing --open`, or `cargo check -p primitive_generation --no-default-features --features text` to see the surface shrink.

## Usage

### Basic Primitive Generation

```rust,no_run
use primitive_generation::{PrimitiveData, AttributesData, Transform};
use std::cell::RefCell;
use std::rc::Rc;
use minwebgl::F32x4;

// Create basic geometry data
let attributes = AttributesData {
  positions: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]],
  indices: vec![0, 1, 2],
};

let primitive = PrimitiveData {
  name: None,
  parent: None,
  attributes: Some(Rc::new(RefCell::new(attributes))),
  color: F32x4::from_array([1.0, 0.0, 0.0, 1.0]),
  transform: Transform::default(),
};
```

### Curve to Ribbon Mesh

```rust,no_run
use primitive_generation::curve_to_geometry;

// A 2D polyline becomes a triangulated ribbon 0.1 units wide.
let ribbon = curve_to_geometry( &[ [ 0.0, 0.0 ], [ 1.0, 0.5 ], [ 2.0, 0.0 ] ], 0.1 );
assert!( ribbon.is_some() );
```

### Text to 3D Geometry

Requires the `font-processing` feature; fonts load from `static/fonts/ufo/<name>.ufo`.

```rust,no_run
use primitive_generation::{ text, Transform };

async fn text_to_meshes() -> usize
{
  let fonts = text::ufo::load_fonts( &[ "main_font" ] ).await;
  text::ufo::text_to_mesh( "hello", &fonts[ "main_font" ], &Transform::default() ).len()
}
# fn main() { let _ = text_to_meshes; }
```

See `examples/minwebgl/text_rendering` for the full pipeline rendered in a browser.

## Platform Support

This crate supports multiple platforms:

- `wasm32-unknown-unknown` (WebAssembly)
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `x86_64-apple-darwin`

## Dependencies

- `minwebgl`: WebGL context management
- `mingl`: 3D mathematics utilities
- `renderer`: GLTF scene types that `primitives_data_to_gltf` assembles into
- `kurbo`: vector path flattening (`text`)
- `norad` + `quick-xml`: UFO font parsing (`font-processing`)
- `earcutr`: polygon triangulation with holes (`font-processing`)

## License

Licensed under the MIT License. See [LICENSE](./license) file for details.

## Contributing

Contributions are welcome! Please see the [repository](https://github.com/Wandalen/cgtools) for contribution guidelines.
