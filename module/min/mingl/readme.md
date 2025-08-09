# 🎮 mingl

> **Agnostic graphics library providing abstract rendering backend utilities**

A versatile graphics abstraction layer designed to work across different rendering backends. Provides essential utilities for camera controls, data conversion, and graphics primitives that can be used with WebGL, Metal, Vulkan, or other graphics APIs.

## ✨ Features

### 🔄 **Data Conversion**
- **Type-Safe Conversions** - Convert between graphics data types safely
- **Vector Operations** - Support for f32, i8/16/32, u8/16/32 numeric types
- **Array Handling** - 1D and 2D array processing with optimized layouts
- **Byte Slice Utilities** - Efficient conversion to GPU buffer formats

### 📷 **Camera System**
- **Orbital Camera Controller** - Smooth camera orbiting around target points
- **Interactive Controls** - Mouse and keyboard input handling
- **Perspective & Orthographic** - Multiple projection modes
- **View Matrix Management** - Optimized view transformation calculations

### 🛠️ **Rendering Utilities**
- **Object Model Reporting** - Analyze and report on 3D model properties
- **Backend Abstraction** - Work across different graphics APIs
- **Performance Optimized** - Minimal overhead abstractions
- **Memory Management** - Efficient buffer and data handling

## 📦 Installation

Add to your `Cargo.toml`:
```toml
mingl = { workspace = true, features = ["camera_orbit_controls"] }
```

## 🚀 Quick Start

### Camera Controls

```rust,ignore
use mingl::camera_orbit_controls::{Camera, OrbitControls};

fn setup_camera() {
  // Create orbital camera controller
  let mut camera = Camera::new()
    .position([0.0, 0.0, 5.0])
    .target([0.0, 0.0, 0.0])
    .up([0.0, 1.0, 0.0]);
  
  let mut controls = OrbitControls::new()
    .distance(10.0)
    .rotation_speed(0.5)
    .zoom_speed(0.1);
  
  // Update camera based on input
  let delta_time = 0.016; // 60fps
  controls.update(&mut camera, delta_time);
  
  // Get view and projection matrices
  let view_matrix = camera.view_matrix();
  let (aspect_ratio, fov, near, far) = (16.0/9.0, 45.0, 0.1, 100.0);
  let proj_matrix = camera.projection_matrix(aspect_ratio, fov, near, far);
}
```

### Data Conversion

```rust,ignore
use mingl::convert::{IntoVector, IntoBytes};

fn data_conversion_examples() {
  // Convert numeric types to vectors
  let float_data: Vec<f32> = vec![1.0, 2.0, 3.0, 4.0];
  let vector = float_data.into_vector();
  
  // Convert 2D arrays
  let positions = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0], 
    [0.5, 1.0, 0.0],
  ];
  let vertex_buffer = positions.into_bytes();
  
  // Handle different numeric types
  let indices: Vec<u16> = vec![0, 1, 2];
  let index_buffer = indices.into_bytes();
}
```

## 📖 API Reference

### Core Components

| Component | Purpose | Key Methods |
|-----------|---------|-------------|
| `Camera` | 3D camera management | `position()`, `look_at()`, `view_matrix()` |
| `OrbitControls` | Interactive camera controls | `update()`, `distance()`, `rotation_speed()` |
| `ToVector` | Type conversion trait | `to_vector()` |
| `ToBytes` | Buffer conversion trait | `to_bytes()` |

### Data Conversion Support

| Type | Vector Support | Bytes Support | Use Case |
|------|---------------|---------------|----------|
| `f32` | ✅ | ✅ | Vertex positions, colors |
| `i8/i16/i32` | ✅ | ✅ | Signed integer data |
| `u8/u16/u32` | ✅ | ✅ | Indices, unsigned data |
| `[T; N]` | ✅ | ✅ | Fixed-size arrays |
| `Vec<T>` | ✅ | ✅ | Dynamic arrays |

### Camera Configuration

```rust,ignore
use mingl::camera_orbit_controls::*;

// Configure orbital camera
let (x, y, z) = (0.0, 0.0, 5.0);
let (tx, ty, tz) = (0.0, 0.0, 0.0);
let (ux, uy, uz) = (0.0, 1.0, 0.0);
let camera = Camera::new()
  .position([x, y, z])
  .target([tx, ty, tz])
  .up([ux, uy, uz])
  .fov(60.0)
  .near(0.1)
  .far(100.0);

// Setup orbit controls
let controls = OrbitControls::new()
  .distance(10.0)           // Distance from target
  .rotation_speed(1.0)      // Rotation sensitivity
  .zoom_speed(0.2)          // Zoom sensitivity
  .min_distance(1.0)        // Closest zoom
  .max_distance(50.0)       // Farthest zoom
  .enable_damping(true);    // Smooth movement
```

## 🎯 Use Cases

### Game Development
- **3D Scene Navigation** - Interactive camera controls for exploring scenes
- **Asset Loading** - Convert model data for GPU upload
- **Input Handling** - Abstract input processing across platforms

### Graphics Applications
- **CAD Viewers** - Precise camera controls for technical drawings
- **Data Visualization** - Navigate complex 3D data sets
- **Scientific Visualization** - Examine 3D models and simulations

### Cross-Platform Development
- **Backend Abstraction** - Write once, run on multiple graphics APIs
- **Performance Optimization** - Efficient data conversion and management
- **Prototype Development** - Rapid graphics application prototyping

## 🔧 Advanced Features

### Custom Camera Controllers

```rust,ignore
use mingl::camera_orbit_controls::*;

struct CustomController {
  sensitivity: f32,
  momentum: Vec3,
}

impl CameraController for CustomController {
  fn update(&mut self, camera: &mut Camera, input: &InputState, dt: f32) {
    // Custom camera control logic
    if input.mouse_down(MouseButton::Left) {
      let delta = input.mouse_delta();
      camera.rotate_around_target(delta.x * self.sensitivity, delta.y * self.sensitivity);
    }
  }
}
```

### Efficient Data Processing

```rust,ignore
use mingl::convert::*;

// Batch convert vertex data efficiently
fn process_mesh_data(vertices: &[[f32; 3]], normals: &[[f32; 3]], uvs: &[[f32; 2]]) -> Vec<u8> {
  let mut buffer = Vec::new();
  
  // Interleave vertex attributes for optimal GPU access
  for i in 0..vertices.len() {
    buffer.extend_from_slice(&vertices[i].into_bytes());
    buffer.extend_from_slice(&normals[i].into_bytes());
    buffer.extend_from_slice(&uvs[i].into_bytes());
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
use mingl::camera_orbit_controls::*;
use mingl::convert::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer};

fn setup_webgl_scene(gl: &WebGl2RenderingContext) {
  let camera = Camera::new().position([0.0, 0.0, 5.0]);
  
  // Convert vertex data for WebGL
  let vertices = vec![[0.0, 1.0, 0.0], [-1.0, -1.0, 0.0], [1.0, -1.0, 0.0]];
  let vertex_buffer = vertices.into_bytes();
  
  // Upload to GPU
  let buffer = gl.create_buffer().unwrap();
  gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));
  gl.buffer_data_with_u8_array(WebGl2RenderingContext::ARRAY_BUFFER, &vertex_buffer, WebGl2RenderingContext::STATIC_DRAW);
}
```

## 📚 Technical Architecture

### Backend Agnostic Design
The library uses trait-based abstractions to ensure compatibility across different graphics backends while maintaining zero-cost abstractions where possible.

### Type Safety
Strong typing prevents common graphics programming errors like incorrect buffer formats or incompatible data conversions.

### Performance Focus
All conversions and operations are designed to minimize CPU overhead and memory allocations in performance-critical rendering loops.
