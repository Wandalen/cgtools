# primitive_generation

3D geometry generation toolkit with primitives, text rendering, and procedural shape creation.

[![Crates.io](https://img.shields.io/crates/v/primitive_generation.svg)](https://crates.io/crates/primitive_generation)
[![Documentation](https://docs.rs/primitive_generation/badge.svg)](https://docs.rs/primitive_generation)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Features

- **3D Primitives**: Generate spheres, cubes, cylinders, and other basic shapes
- **Text Rendering**: Convert text to 3D geometry with font support
- **Procedural Generation**: Create complex shapes algorithmically
- **CSG Operations**: Constructive Solid Geometry for shape combinations
- **Font Processing**: Advanced typography and text layout capabilities
- **glTF Import**: Load and process 3D models from glTF files
- **WebAssembly Ready**: Optimized for browser environments

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

## Features

- `enabled` (default): Core geometry generation functionality
- `full`: All features enabled
- `csg`: Constructive Solid Geometry operations
- `text`: Text rendering and font processing
- `font-processing`: Advanced font processing features
- `gltf-import`: glTF model loading support
- `random`: Random geometry generation

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
  attributes: Rc::new(RefCell::new(attributes)),
  color: F32x4::from_array([1.0, 0.0, 0.0, 1.0]),
  transform: Transform::default(),
};
```

### Text to 3D Geometry

```rust,no_test
// Text rendering functionality is planned but not yet implemented
// This feature will be available in future versions
```

### CSG Operations

```rust,no_test
// CSG operations are planned but not yet implemented
// This feature will be available in future versions
```

## Platform Support

This crate supports multiple platforms:

- `wasm32-unknown-unknown` (WebAssembly)
- `x86_64-unknown-linux-gnu`
- `x86_64-pc-windows-msvc`
- `x86_64-apple-darwin`

## Dependencies

- `minwebgl`: WebGL context management
- `mingl`: 3D mathematics utilities
- `renderer`: Core rendering support
- `gltf`: 3D model loading (optional)
- `csgrs`: CSG operations (optional)
- `kurbo`: Vector graphics (optional)

## License

Licensed under the MIT License. See [LICENSE](license) file for details.

## Contributing

Contributions are welcome! Please see the [repository](https://github.com/Wandalen/cgtools) for contribution guidelines.
