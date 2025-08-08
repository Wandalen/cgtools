# 🎯 ndarray_cg

> **High-performance computer graphics mathematics built on ndarray**

A comprehensive matrix and linear algebra library specifically designed for computer graphics applications. Built on top of the powerful `ndarray` crate, ndarray_cg provides specialized functionality for 2D/3D transformations, graphics pipelines, and geometric operations with excellent performance characteristics.

## ✨ Features

### 🔄 **Matrix Operations**
- **2D/3D/4D Matrices** - Specialized matrix types for graphics (Mat2, Mat3, Mat4)
- **Row & Column Major** - Support for both memory layouts
- **Operator Overloading** - Natural mathematical syntax (+, -, *, etc.)
- **In-Place Operations** - Memory-efficient computations

### 🎮 **Graphics Transformations**
- **Rotation Matrices** - 2D/3D rotation operations
- **Translation** - Homogeneous coordinate translations
- **Scaling** - Uniform and non-uniform scaling
- **Reflection** - Mirror transformations
- **Shearing** - Skew transformations

### 🚀 **Performance Optimized**
- **SIMD Support** - Vectorized operations via ndarray
- **Zero-Copy Views** - Efficient memory access patterns
- **Stack Allocation** - Small matrices on stack
- **Generic Types** - Works with f32, f64, and other numeric types

### 🔧 **Developer Experience**
- **Type Aliases** - Convenient F32x2x2, F32x3x3, F32x4x4 types
- **Indexed Iteration** - Multiple iteration patterns
- **Debug Support** - Rich debugging and display traits

## 📦 Installation

Add to your `Cargo.toml`:
```toml
ndarray_cg = { workspace = true, features = ["enabled"] }
```

## 🚀 Quick Start

### Basic Matrix Operations

```rust
use ndarray_cg::*;

fn main() {
  // Create a 2x2 matrix using row-major layout
  // [ 1.0, 2.0 ]
  // [ 3.0, 4.0 ]
  let matrix = Mat2::from_row_major([1.0, 2.0, 3.0, 4.0]);
  
  // Create a rotation matrix (45 degrees)
  let rotation = mat2x2::rot(45.0f32.to_radians());
  
  // Apply rotation to the matrix
  let rotated = rotation * matrix;
  
  println!("Original: {:?}", matrix.raw_slice());
  println!("Rotated:  {:?}", rotated.raw_slice());
}
```

### Common Graphics Transformations

```rust
use ndarray_cg::*;
use std::f32::consts::PI;

fn graphics_example() {
  // 2D Rotation (45 degrees)
  let angle = PI / 4.0;
  let rotation = mat2x2::rot(angle);
  
  // 2D Scaling
  let scale = mat2x2::scale([2.0, 3.0]);
  
  // 2D Translation (using homogeneous coordinates)
  let translation = mat2x2h::translate([5.0, 10.0]);
  
  // Reflection across X-axis
  let reflect = mat2x2::reflect_x();
  
  // Shearing transformation
  let shear = mat2x2::shear([1.0, 0.5]);
  
  // Combine transformations
  let transform = translation * scale * rotation;
  println!("Combined transform: {:?}", transform.raw_slice());
}
```

### Matrix Arithmetic Operations

```rust
use ndarray_cg::*;

fn arithmetic_examples() {
  // Matrix addition
  let mat_a = F32x2x2::from_row_major([
    1.0, 2.0,
    3.0, 4.0,
  ]);
  let mat_b = F32x2x2::from_row_major([
    5.0, 6.0,
    7.0, 8.0,
  ]);
  
  // Using operator overloading (recommended)
  let result = &mat_a + &mat_b;
  
  // Or using explicit function calls
  let mut result2 = F32x2x2::default();
  d2::add(&mut result2, &mat_a, &mat_b);
  
  println!("Addition result: {:?}", result.raw_slice());
}
```

### Matrix Multiplication

```rust
use ndarray_cg::*;
use mat::DescriptorOrderRowMajor;

fn multiplication_example() {
  // 1x3 matrix
  let mat_a = Mat::<1, 3, f32, DescriptorOrderRowMajor>::from_row_major([
    1.0, 2.0, 3.0,
  ]);
  
  // 3x2 matrix  
  let mat_b = Mat::<3, 2, f32, DescriptorOrderRowMajor>::from_row_major([
    7.0, 8.0,
    9.0, 10.0,
    11.0, 12.0,
  ]);
  
  // Matrix multiplication: 1x3 * 3x2 = 1x2
  let result = &mat_a * &mat_b;
  println!("Multiplication result: {:?}", result.raw_slice()); // [58.0, 64.0]
}
```

## 📖 API Reference

### Matrix Types

| Type | Description | Use Case |
|------|-------------|----------|
| `Mat2` / `F32x2x2` | 2x2 matrix | 2D transformations |
| `Mat3` / `F32x3x3` | 3x3 matrix | 2D homogeneous coords |
| `Mat4` / `F32x4x4` | 4x4 matrix | 3D transformations |

### Transformation Functions

| Function | Purpose | Example |
|----------|---------|---------|
| `mat2x2::rot(angle)` | 2D rotation | `mat2x2::rot(PI / 4.0)` |
| `mat2x2::scale([sx, sy])` | 2D scaling | `mat2x2::scale([2.0, 3.0])` |
| `mat2x2h::translate([tx, ty])` | 2D translation | `mat2x2h::translate([5.0, 10.0])` |
| `mat2x2::reflect_x()` | X-axis reflection | `mat2x2::reflect_x()` |
| `mat2x2::shear([shx, shy])` | Shearing | `mat2x2::shear([1.0, 0.5])` |

### Advanced Features

```rust
use ndarray_cg::*;
use mat::DescriptorOrderRowMajor;

fn advanced_features() {
  // Matrix iteration
  let matrix = Mat::<1, 2, f32, DescriptorOrderRowMajor>::from_row_major([1.0, 2.0]);
  
  // Iterate over matrix lanes (rows/columns)
  for value in matrix.lane_iter(0, 0) {
    println!("Value: {}", value);
  }
  
  // Indexed iteration
  for (index, value) in matrix.iter_indexed_msfirst() {
    println!("Index: {:?}, Value: {}", index, value);
  }
}
```

## 🎯 Use Cases

- **2D Game Development** - Sprite transformations and camera systems
- **3D Graphics Programming** - Model-view-projection matrices
- **Computer Vision** - Image transformations and homography
- **Animation Systems** - Skeletal animation and keyframe interpolation
- **Physics Simulations** - Coordinate system transformations

## 🔧 Advanced Configuration

### Features

- `enabled` - Core functionality (default)
- `full` - All features enabled

### Memory Layout Options

```rust
use ndarray_cg::*;
use mat::{DescriptorOrderRowMajor, DescriptorOrderColMajor};

// Row-major (default for graphics)
let row_major = Mat::<2, 2, f32, DescriptorOrderRowMajor>::from_row_major([1.0, 2.0, 3.0, 4.0]);

// Column-major (for compatibility with certain graphics APIs)
let col_major = Mat::<2, 2, f32, DescriptorOrderColMajor>::default();
```

## ⚡ Performance Notes

- Built on ndarray's highly optimized SIMD operations
- Zero-cost abstractions over raw array operations
- Efficient memory layouts for cache performance
- Generic over numeric types (f32, f64, etc.)
