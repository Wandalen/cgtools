# 🎮 mingl

> **Agnostic graphics library providing abstract rendering backend utilities**

A versatile graphics abstraction layer designed to work across different rendering backends. Provides essential utilities for camera controls, GPU data type descriptors, and model analysis that can be used with WebGL, WebGPU, or other graphics APIs.

## ✨ Features

### 🔄 **Data Type Descriptors & Byte Conversion**
- **`VectorDataType` Descriptors** - Describe the scalar type, atom count, and element count of GPU attribute data
- **Primitive Coverage** - `f32`, `i8/i16/i32`, `u8/u16/u32` scalars, fixed-size arrays `[T; N]`, and nested arrays `[[T; M]; N]` for all supported scalars
- **`IntoVectorDataType` Trait** - Map Rust types to descriptors for backend-agnostic attribute setup
- **Byte Slice Utilities** - Re-exported `AsBytes`/`IntoBytes` traits (from the `asbytes` crate) for `Pod`-safe conversion to GPU buffer bytes

### 📷 **Camera System**
- **Orbital Camera Controller** - `CameraOrbitControls`: rotate, pan, and zoom around a target point (`camera_orbit_controls` feature)
- **Mouse & Touch Input** - `controls_bind_to_input` wires pointer events — drag to rotate, right-drag to pan, wheel/pinch to zoom (`web` feature)
- **Motion Constraints** - Longitude/latitude rotation ranges, min/max zoom distance, optional movement smoothing with decay
- **View Matrix** - Right-handed `view()` matrix computed from the camera's current state

### 🛠️ **More Utilities**
- **Object Model Reporting** - Bounding box/sphere and memory-size reports for OBJ models (`model_obj` feature)
- **Character Controller** - First-person style `CharacterControls` with yaw/pitch and planar movement vectors (`character_controls` feature)
- **Web Helpers** - Canvas/DOM setup, `requestAnimationFrame` exec loop, file fetch, logging (`web` feature)
- **Backend Abstraction** - Descriptor-based design usable from WebGL, WebGPU, or other APIs

## 📦 Installation

Add to your `Cargo.toml`:
```toml
mingl = { workspace = true, features = ["camera_orbit_controls"] }
```

## 🚀 Quick Start

### Camera Controls

```rust,ignore
use mingl::CameraOrbitControls;

fn setup_camera()
{
  // Orbital camera controller — public fields + Default
  let mut camera = CameraOrbitControls
  {
    eye : [ 0.0, 0.0, 5.0 ].into(),
    center : [ 0.0, 0.0, 0.0 ].into(),
    window_size : [ 1280.0, 720.0 ].into(),
    ..Default::default()
  };

  // Feed screen-space input deltas
  camera.rotate( [ 10.0, 0.0 ] ); // drag
  camera.pan( [ 0.0, 5.0 ] );     // right-drag
  camera.zoom( -120.0 );          // wheel

  // Advance movement smoothing each frame
  camera.update( 16.0 );

  // Right-handed view matrix for rendering
  let view_matrix = camera.view();
}
```

On the web, `controls_bind_to_input( &canvas, &camera )` (`web` feature, `camera : Rc<RefCell<CameraOrbitControls>>`) wires
mouse drag/right-drag/wheel and touch drag/pinch to these methods for you. Projection is out of scope — build a
projection matrix with `ndarray_cg` (re-exported as `mingl::math` under the `math` feature).

### Data Descriptors & Byte Conversion

```rust,ignore
use mingl::{ IntoVectorDataType, VectorDataType, DataType, IntoBytes };

fn data_conversion_examples()
{
  // Describe an attribute's layout backend-agnostically
  let desc : VectorDataType = < [ f32; 3 ] >::into_vector_data_type();
  assert_eq!( desc.scalar, DataType::F32 );
  assert_eq!( desc.natoms, 3 );

  // Convert 2D arrays to GPU-ready bytes ( `Pod`-based, via re-exported `asbytes` )
  let positions =
  [
    [ 0.0, 0.0, 0.0 ],
    [ 1.0, 0.0, 0.0 ],
    [ 0.5, 1.0, 0.0 ],
  ];
  let vertex_buffer = positions.into_bytes();

  // Handle different numeric types
  let indices : Vec< u16 > = vec![ 0, 1, 2 ];
  let index_buffer = indices.into_bytes();
}
```

## 📖 API Reference

### Core Components

| Component | Purpose | Key Methods |
|-----------|---------|-------------|
| `CameraOrbitControls` | Orbit camera around a target point | `rotate()`, `pan()`, `zoom()`, `update()`, `view()` |
| `CameraRotationState` / `CameraZoomState` / `CameraPanState` | Per-motion constraints and sensitivity (`camera.rotation` / `.zoom` / `.pan` fields) | `longitude_range_set()`, `min_distance_set()`, `movement_decay_set()` |
| `controls_bind_to_input` | Wire canvas pointer events to a camera (`web` feature) | mouse drag/right-drag/wheel, touch drag/pinch |
| `IntoVectorDataType` | Map Rust types to attribute descriptors | `into_vector_data_type()` |
| `VectorDataType` | Attribute layout descriptor | `byte_size()`, `natoms()`, `scalar()` |
| `AsBytes` / `IntoBytes` | Buffer conversion traits (re-exported `asbytes`) | `as_bytes()`, `byte_size()`, `into_bytes()` |

### Data Conversion Support

| Type | Descriptor (`IntoVectorDataType`) | Bytes (`IntoBytes`) | Use Case |
|------|-----------------------------------|---------------------|----------|
| `f32` | ✅ scalar, `[f32; N]`, nested `[[f32; M]; N]` | ✅ | Vertex positions, colors |
| `i8/i16/i32` | ✅ scalar and `[T; N]` | ✅ | Signed integer data |
| `u8/u16/u32` | ✅ scalar and `[T; N]` | ✅ | Indices, unsigned data |
| `Vec<T>` | ❌ descriptors are compile-time — use `[T; N]` | ✅ | Dynamic arrays (bytes only) |

### Camera Configuration

```rust,ignore
use mingl::CameraOrbitControls;

let mut camera = CameraOrbitControls::default();

// Constrain rotation ( angles in degrees; setters clamp to valid ranges )
camera.rotation.base_longitude_set( 0.0 );   // clamped to [ 0, 360 ]
camera.rotation.longitude_range_set( 90.0 ); // clamped to [ 0, 180 ]
camera.rotation.base_latitude_set( 0.0 );    // clamped to [ -90, 90 ]
camera.rotation.latitude_range_set( 60.0 );  // clamped to [ 0, 180 ]

// Zoom limits and sensitivity ( larger speed = slower motion )
camera.zoom.min_distance_set( 1.0 );
camera.zoom.max_distance_set( 50.0 );
camera.zoom.speed = 1000.0;
camera.rotation.speed = 500.0;

// Smooth rotation with decay, or disable a motion entirely
camera.rotation.movement_smoothing_enabled = true;
camera.rotation.movement_decay_set( 0.05 );
camera.pan.enabled = false;
```

## 🎯 Use Cases

### Game Development
- **3D Scene Navigation** - Interactive camera controls for exploring scenes
- **Asset Analysis** - Bounding volumes and memory reports for loaded OBJ models
- **Input Handling** - Pointer-driven camera input on web canvases

### Graphics Applications
- **CAD Viewers** - Precise camera controls for technical drawings
- **Data Visualization** - Navigate complex 3D data sets
- **Scientific Visualization** - Examine 3D models and simulations

### Cross-Platform Development
- **Backend Abstraction** - Write once, run on multiple graphics APIs
- **Performance Optimization** - Efficient data conversion and management
- **Prototype Development** - Rapid graphics application prototyping

## 🔧 Advanced Features

### Character Controller

```rust,ignore
use mingl::CharacterControls;

// First-person style controller ( `character_controls` feature )
fn read_pose( controls : &CharacterControls )
{
  let position = controls.position();
  let ( yaw, pitch ) = ( controls.yaw(), controls.pitch() );
  // Movement vectors projected on the XZ plane
  let forward = controls.forward_xz();
  let right = controls.right_xz();
}
```

### OBJ Model Reporting

```rust,ignore
use mingl::model::obj::{ BoundingBox, BoundingSphere };

// Analyze model geometry ( `model_obj` feature )
fn analyze( positions : &[ f32 ] )
{
  let bounding_box = BoundingBox::compute( positions );
  let bounding_sphere = BoundingSphere::compute( positions, &bounding_box );
}
```

### Efficient Data Processing

```rust,ignore
use mingl::IntoBytes;

// Batch convert vertex data efficiently
fn process_mesh_data( vertices : &[ [ f32; 3 ] ], normals : &[ [ f32; 3 ] ], uvs : &[ [ f32; 2 ] ] ) -> Vec< u8 >
{
  let mut buffer = Vec::new();

  // Interleave vertex attributes for optimal GPU access
  for i in 0..vertices.len()
  {
    buffer.extend_from_slice( &vertices[ i ].into_bytes() );
    buffer.extend_from_slice( &normals[ i ].into_bytes() );
    buffer.extend_from_slice( &uvs[ i ].into_bytes() );
  }

  buffer
}
```

## ⚡ Performance Considerations

### Memory Efficiency
- Minimize allocations with in-place conversions where possible
- Use appropriate buffer sizes for GPU upload
- Cache frequently accessed transformation matrices

### CPU Optimization
- Batch data conversions to reduce function call overhead
- Use SIMD-friendly data layouts when possible
- Profile camera update frequency for optimal performance

## 🔧 Integration Examples

### With WebGL
```rust,ignore
use mingl::{ CameraOrbitControls, IntoVectorDataType, IntoBytes };
use web_sys::WebGl2RenderingContext;

fn setup_webgl_scene( gl : &WebGl2RenderingContext )
{
  let camera = CameraOrbitControls
  {
    eye : [ 0.0, 0.0, 5.0 ].into(),
    ..Default::default()
  };

  // Describe the attribute layout backend-agnostically
  let desc = < [ f32; 3 ] >::into_vector_data_type();
  let component_count = desc.natoms(); // 3 — for vertex_attrib_pointer

  // Convert vertex data for WebGL
  let vertices = vec![ [ 0.0, 1.0, 0.0 ], [ -1.0, -1.0, 0.0 ], [ 1.0, -1.0, 0.0 ] ];
  let vertex_buffer = vertices.into_bytes();

  // Upload to GPU
  let buffer = gl.create_buffer().unwrap();
  gl.bind_buffer( WebGl2RenderingContext::ARRAY_BUFFER, Some( &buffer ) );
  gl.buffer_data_with_u8_array( WebGl2RenderingContext::ARRAY_BUFFER, &vertex_buffer, WebGl2RenderingContext::STATIC_DRAW );
}
```

## 📚 Technical Architecture

### Backend Agnostic Design
The library uses trait-based abstractions to ensure compatibility across different graphics backends while maintaining zero-cost abstractions where possible.

### Type Safety
Strong typing prevents common graphics programming errors like incorrect buffer formats or incompatible data conversions.

### Performance Focus
All conversions and operations are designed to minimize CPU overhead and memory allocations in performance-critical rendering loops.
