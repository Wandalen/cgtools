# 🧮 mdmath_core

> **Fundamental multidimensional mathematics for computer graphics and scientific computing**

A high-performance, type-safe mathematics library providing essential vector operations and geometric primitives. Built specifically for computer graphics applications with support for n-dimensional vector spaces and optimized operations.

## ✨ Features

### 🔢 **Vector Operations**
- **Dot Product** - Efficient vector dot product calculations
- **Magnitude & Normalization** - Vector length and unit vector operations  
- **Projection** - Project vectors onto other vectors
- **Angular Calculations** - Compute angles between vectors
- **Orthogonality Testing** - Check perpendicular relationships
- **Dimension Handling** - N-dimensional vector support

### 🛠️ **Memory Management**
- **Zero-Copy Operations** - Efficient slice and tuple conversions
- **Mutable References** - Safe in-place vector modifications
- **Iterator Support** - Standard Rust iteration patterns
- **Type Safety** - Compile-time guarantees for vector operations

### 🚀 **Performance**
- **SIMD Optimization** - Vectorized operations where possible
- **Stack Allocation** - Minimal heap usage for small vectors
- **Generic Implementation** - Works with any numeric type

## 📦 Installation

Add to your `Cargo.toml`:
```toml
mdmath_core = { workspace = true }
```

Or with specific features:
```toml
mdmath_core = { workspace = true, features = ["full"] }
```

## 🚀 Quick Start

### Basic Vector Operations

```rust,ignore
use mdmath_core::vector;

fn main() {
  // Create vectors as arrays
  let vec_a = [1.0, 2.0, 3.0];
  let vec_b = [4.0, 5.0, 6.0];
  
  // Dot product
  let dot_result = vector::dot(&vec_a, &vec_b);
  println!("Dot product: {}", dot_result); // 32.0
  
  // Vector magnitude
  let magnitude: f32 = vector::mag2(&vec_a);
  let magnitude = magnitude.sqrt();
  println!("Magnitude: {}", magnitude); // ~3.74
  
  // Normalize vector
  let mut normalized = vec_a;
  vector::normalize(&mut normalized, &vec_a);
  println!("Normalized: {:?}", normalized);
}
```

### Advanced Vector Operations

```rust,ignore
use mdmath_core::vector;
use approx::assert_ulps_eq;

fn advanced_example() {
  // Vector projection
  let mut vec_a = [1.0, 2.0, 3.0];
  let vec_b = [4.0, 5.0, 6.0];
  vector::project_on(&mut vec_a, &vec_b);
  
  // Angle between vectors
  let vec_x = [1.0, 0.0];
  let vec_y = [0.0, 1.0];
  let angle = vector::angle(&vec_x, &vec_y);
  assert_ulps_eq!(angle, std::f32::consts::FRAC_PI_2);
  
  // Check orthogonality
  let is_orthogonal = vector::is_orthogonal(&vec_x, &vec_y);
  assert!(is_orthogonal);
}
```

## 📖 API Reference

### Core Functions

| Function | Description | Example |
|----------|-------------|---------|
| `dot(a, b)` | Compute dot product | `vector::dot(&[1,2], &[3,4])` |
| `mag2(v)` | Squared magnitude | `vector::mag2(&[3,4])` → `25.0` |
| `normalize(dst, src)` | Normalize vector | `vector::normalize(&mut v, &src)` |
| `project_on(a, b)` | Project a onto b | `vector::project_on(&mut a, &b)` |
| `angle(a, b)` | Angle between vectors | `vector::angle(&a, &b)` |
| `is_orthogonal(a, b)` | Check perpendicularity | `vector::is_orthogonal(&a, &b)` |

### Features

Enable additional functionality:
```toml
mdmath_core = { workspace = true, features = ["full", "approx", "arithmetics"] }
```

- `full` - All features enabled
- `approx` - Floating-point comparison utilities  
- `arithmetics` - Advanced arithmetic operations
- `nd` - N-dimensional array support

## 🎯 Use Cases

- **Computer Graphics** - 3D transformations and lighting calculations
- **Game Development** - Physics simulations and collision detection
- **Scientific Computing** - Mathematical modeling and analysis
- **Machine Learning** - Vector operations for neural networks
- **Robotics** - Spatial calculations and motion planning

## ⚡ Performance

mdmath_core is designed for high-performance applications:
- Zero-allocation operations on stack arrays
- Generic implementations work with any numeric type
- Optimized for common vector sizes (2D, 3D, 4D)
- SIMD optimizations where available
