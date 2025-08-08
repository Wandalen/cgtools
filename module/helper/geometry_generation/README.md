# Geometry generation

The `geometry_generation` crate is a powerful tool for programmatically creating 3D geometry from abstract data and converting it into a renderable GLTF scene graph. It includes modules for handling 3D primitives, curves, and text, making it ideal for on-the-fly geometry creation in graphics applications.

-----

## How It Works

This crate is structured into three main layers: `text`, `primitive`, and `primitive_data`.

  * **`primitive_data`:** This is the core module that defines the foundational data structures for 3D geometry. It includes `Transform` for manipulating an object's position, rotation, and scale, and `PrimitiveData` for storing vertex attributes and indices. The key function here is `primitives_data_to_gltf`, which takes a collection of `PrimitiveData` and builds a complete `GLTF` scene. This GLTF object is ready to be rendered by a WebGL renderer, as it includes all the necessary buffers, geometries, and scene nodes.
  * **`primitive`:** This layer contains functions for generating `PrimitiveData` from higher-level abstractions. A notable function is `curve_to_geometry`, which takes a 2D curve and converts it into a 3D polygonal representation (a series of rectangles) with a specified thickness. This allows you to easily render 2D shapes in a 3D scene.
  * **`text`:** This module is designed for handling font loading and converting text into a 3D mesh. It includes a sub-module for processing `.ufo` font files. This functionality is crucial for dynamic text rendering in a 3D environment, such as creating labels, titles, or other text-based UI elements.

-----

## How to Use It

The primary use case for this crate is to create renderable 3D geometry from a variety of sources. Here is a simple example of how to create a 3D curve and a GLTF scene from it:

1.  **Generate `PrimitiveData` for the curve.**

Use the `curve_to_geometry` function to create a `PrimitiveData` object from a list of 2D points.

```rust
use geometry_generation::primitive::curve_to_geometry;

let points = [ [ 0.0, 0.0 ], [ 1.0, 1.0 ], [ 2.0, 0.0 ] ];
let curve_primitive = curve_to_geometry( &points, 0.1 );
```

2.  **Convert `PrimitiveData` to a GLTF scene.**

Pass the generated `PrimitiveData` to `primitives_data_to_gltf` to create a `GLTF` object that can be rendered.

```rust
use geometry_generation::primitive_data::primitives_data_to_gltf;
use renderer::webgl::loaders::gltf::GLTF;

// `gl` is a WebGL2RenderingContext instance
let gltf_scene: GLTF = primitives_data_to_gltf( &gl, vec![ curve_primitive.unwrap() ] );
```

This `gltf_scene` can now be used with a compatible renderer to display the curve in your WebGL application. The same pattern applies to text and other primitives, providing a flexible and powerful way to generate dynamic content.