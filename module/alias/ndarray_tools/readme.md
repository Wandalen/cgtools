# 🧮 ndarray_tools

**Math for Computer Graphics with ndarray**

A powerful mathematical toolkit for computer graphics built on top of the versatile `ndarray` crate. This alias module provides a flexible and performant approach to CG mathematics, leveraging ndarray's robust numerical computing capabilities while maintaining compatibility with specialized graphics-focused crates.

## 🎯 Features

- **High Performance**: Built on `ndarray` for efficient numerical operations
- **Flexible Architecture**: Adaptable to various computer graphics workflows
- **Comprehensive Math**: Complete suite of mathematical operations for CG
- **Memory Efficient**: Optimized memory usage patterns
- **Type Safety**: Rust's type system ensures mathematical correctness
- **Interoperability**: Compatible with other graphics and game development crates

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
ndarray_tools = "0.1.0"
```

## 💡 Usage

```rust,ignore
use ndarray_tools::*;

// Basic vector and matrix operations
let vector = Array1::<f32>::zeros(3);
let matrix = Array2::<f32>::eye(4);

// Perform computer graphics calculations
let transformed = matrix.dot(&vector.view());
```

## 📚 Core Capabilities

- **Linear Algebra**: Vectors, matrices, and transformations
- **Geometric Operations**: Points, normals, and coordinate transformations  
- **Numerical Computing**: Efficient array operations and mathematical functions
- **Graphics Pipeline**: Support for vertex processing and matrix operations

## 🔧 Architecture

This crate serves as an alias module that re-exports the core `ndarray_cg` functionality, providing a clean and focused interface for computer graphics mathematical operations.

## 🎮 Use Cases

- **Game Development**: Transform calculations and physics simulations
- **Computer Graphics**: Rendering pipelines and geometric processing
- **Scientific Visualization**: Data visualization and mathematical modeling
- **Animation Systems**: Keyframe interpolation and transformation chains

## 📖 Documentation

For detailed API documentation and examples, run:

```bash
cargo doc --open
```
