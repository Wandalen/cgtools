# geometry_generation

3D geometry generation toolkit with primitives, text rendering, and procedural shape creation.

[![Crates.io](https://img.shields.io/crates/v/geometry_generation.svg)](https://crates.io/crates/geometry_generation)
[![Documentation](https://docs.rs/geometry_generation/badge.svg)](https://docs.rs/geometry_generation)
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
geometry_generation = "0.1.0"
```

For full functionality, enable all features:

```toml
[dependencies]
geometry_generation = { version = "0.1.0", features = ["full"] }
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

```rust
use geometry_generation::primitive::{Sphere, Cube, PrimitiveData};

// Generate a sphere
let sphere = Sphere::new(1.0, 32, 16);
let sphere_data = sphere.generate();

// Generate a cube
let cube = Cube::new(2.0, 2.0, 2.0);
let cube_data = cube.generate();

// Access vertex data
println!("Vertices: {:?}", sphere_data.positions);
println!("Normals: {:?}", sphere_data.normals);
println!("UVs: {:?}", sphere_data.uvs);
```

### Text to 3D Geometry

```rust
// Requires "text" feature
use geometry_generation::text::TextMesh;

let text_mesh = TextMesh::new("Hello World", font_data)?;
let geometry = text_mesh.generate_3d(extrusion_depth)?;
```

### CSG Operations

```rust
// Requires "csg" feature
use geometry_generation::csg::{union, intersection, difference};

let result = union(&cube_mesh, &sphere_mesh)?;
let carved = difference(&cube_mesh, &sphere_mesh)?;
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
