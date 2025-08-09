# 🎨 CGTools - Web-First Computer Graphics Toolkit

[![CI](https://github.com/Wandalen/cgtools/workflows/CI/badge.svg)](https://github.com/Wandalen/cgtools/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![WASM](https://img.shields.io/badge/WebAssembly-Ready-brightgreen)](https://webassembly.org/)

> **Modern, performant computer graphics tools built for the web platform**

CGTools is a comprehensive Rust-based toolkit for computer graphics programming, specifically designed for WebAssembly and web deployment. It provides everything you need to build interactive graphics applications, games, and visualizations that run natively in the browser with near-native performance.

![Abstract Art](./assets/primitives.jpg)

## ✨ Features

### 🚀 **Web-First Architecture**
- **WebGL 2.0 & WebGPU Support** - Modern graphics APIs for high-performance rendering
- **WebAssembly Optimized** - Lightning-fast execution in browsers
- **Zero-Copy Operations** - Efficient memory management and data transfer
- **Browser Integration** - Seamless DOM, input, and file system integration

### 🧮 **Advanced Mathematics**
- **N-Dimensional Arrays** - Powered by `ndarray` with computer graphics extensions
- **Linear Algebra** - Matrix operations, transformations, and vector math
- **Geometric Primitives** - Points, lines, curves, and complex shapes
- **Spatial Algorithms** - Pathfinding, collision detection, and spatial queries

### 🎮 **Game Development Tools**
- **Tile-Based Systems** - Hexagonal and square grids with pathfinding
- **Field-of-View Algorithms** - Multiple FOV calculation methods
- **Entity Component Systems** - Flexible game architecture patterns
- **Input Handling** - Mouse, keyboard, and touch input management

### 🎯 **Specialized Tools**
- **Vector Graphics** - SVG generation and manipulation
- **Image Processing** - Rasterization, filtering, and format conversion
- **3D Model Loading** - glTF, OBJ, and custom format support
- **Embroidery Patterns** - PEC, PES format reading/writing

## 🚀 Quick Start

### Prerequisites
- **Rust 1.75+** with WebAssembly target: `rustup target add wasm32-unknown-unknown`
- **Web server** for serving examples: `python -m http.server` or `npx serve`
- **Optional**: `wasm-pack` for building WebAssembly modules: `cargo install wasm-pack`

### Try an Example
```bash
# Clone and navigate to the project
git clone https://github.com/Wandalen/cgtools
cd cgtools

# Build and run a WebGL example
cd examples/minwebgl/hexagonal_grid
wasm-pack build --target web --out-dir pkg
# Serve and open http://localhost:8000 in your browser
```

### Use in Your Project
Add to your `Cargo.toml`:
```toml
[dependencies]
minwebgl = "0.2"        # WebGL rendering
tiles_tools = "0.1"     # Tile-based game systems  
ndarray_cg = "0.3"      # Computer graphics math
browser_input = "0.1"   # Input handling
```

## 📦 Core Library Crates

### 🎮 **Game Development**
| Crate | Version | Description | Features |
|-------|---------|-------------|----------|
| **[`tiles_tools`](./module/helper/tiles_tools)** | `0.1.0` | Complete tile-based game toolkit | Hex/Square grids, A* pathfinding, ECS, FOV |

### 🎨 **Graphics & Rendering**
| Crate | Version | Description | Features |
|-------|---------|-------------|----------|
| **[`minwebgl`](./module/min/minwebgl)** | `0.2.0` | Minimal WebGL 2.0 toolkit | Shaders, textures, geometry, utilities |
| **[`minwebgpu`](./module/min/minwebgpu)** | `0.1.0` | Minimal WebGPU toolkit | Compute shaders, modern graphics pipeline |
| **[`renderer`](./module/helper/renderer)** | `0.1.0` | High-level 3D rendering system | Scene graphs, PBR, deferred shading |
| **[`line_tools`](./module/helper/line_tools)** | `0.1.0` | High-performance line rendering | Anti-aliasing, batch processing |
| **[`canvas_renderer`](./module/helper/canvas_renderer)** | `0.1.0` | 2D canvas rendering utilities | Sprites, shapes, image processing |

### 🧮 **Mathematics**  
| Crate | Version | Description | Features |
|-------|---------|-------------|----------|
| **[`ndarray_cg`](./module/math/ndarray_cg)** | `0.3.0` | Computer graphics mathematics | Vectors, matrices, quaternions |
| **[`mdmath_core`](./module/math/mdmath_core)** | `0.3.0` | Multidimensional math core | N-dimensional operations, indexing |

### 🌐 **Web Integration**
| Crate | Version | Description | Features |
|-------|---------|-------------|----------|
| **[`browser_input`](./module/helper/browser_input)** | `0.1.0` | Ergonomic input handling | Keyboard, mouse, touch events |
| **[`browser_log`](./module/helper/browser_log)** | `0.3.0` | WebAssembly logging utilities | Console integration, panic handling |

### 🛠️ **Specialized Tools**
| Crate | Version | Description | Features |
|-------|---------|-------------|----------|
| **[`geometry_generation`](./module/helper/geometry_generation)** | `0.1.0` | 3D geometry and text processing | Mesh generation, font parsing |
| **[`embroidery_tools`](./module/helper/embroidery_tools)** | `0.1.0` | Embroidery pattern tools | PES/PEC format support |
| **[`vectorizer`](./module/helper/vectorizer)** | `0.1.0` | Raster to vector conversion | SVG output, CLI interface |

### 📦 **Convenience Aliases**
| Crate | Version | Description |
|-------|---------|-------------|
| **[`browser_tools`](./module/alias/browser_tools)** | `0.2.0` | Browser development convenience package |
| **[`ndarray_tools`](./module/alias/ndarray_tools)** | `0.1.0` | Mathematics convenience package |

## 🎮 Examples & Demos

Explore our interactive examples to see CGTools in action:

### WebGL Demos
- **[Hexagonal Pathfinding](./examples/minwebgl/hexagonal_grid/)** - Interactive hex grid with A* pathfinding
- **[Deferred Shading](./examples/minwebgl/deferred_shading/)** - Modern 3D rendering pipeline
- **[Text Rendering](./examples/minwebgl/text_rendering/)** - GPU-accelerated text with custom fonts
- **[Wave Function Collapse](./examples/minwebgl/wave_function_collapse/)** - Procedural level generation

### WebGPU Demos  
- **[Deferred Rendering](./examples/minwebgpu/deffered_rendering/)** - Next-gen graphics pipeline
- **[Hello Triangle](./examples/minwebgpu/hello_triangle/)** - WebGPU basics

### Game Systems
- **[Game of Life](./module/helper/tiles_tools/examples/game_of_life.rs)** - Conway's Game of Life on hex/square grids
- **[Tactical RPG](./module/helper/tiles_tools/examples/tactical_rpg.rs)** - Turn-based combat system
- **[Stealth Game](./module/helper/tiles_tools/examples/stealth_game.rs)** - Field-of-view mechanics

## 🏗️ Architecture

CGTools follows a modular, web-first architecture:

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Web Frontend  │    │  WASM Graphics  │    │  Rust Backend   │
│                 │    │                 │    │                 │
│ HTML5 Canvas    │◄──►│ WebGL/WebGPU    │◄──►│ CGTools Modules │
│ DOM Integration │    │ Shaders         │    │ Math & Logic    │
│ Input Events    │    │ Textures        │    │ Data Processing │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

## 🛠️ Development

### Building the Workspace
```bash
# Test all library crates
cargo test --workspace

# Check all crates compile
cargo check --workspace

# Build individual crate for publishing
cargo build -p tiles_tools --release

# Build WebAssembly examples
cd examples/minwebgl/trivial
wasm-pack build --target web --out-dir pkg

# Optimize for production
RUSTFLAGS='-C target-feature=+simd128' cargo build --release --target wasm32-unknown-unknown
```

### Publishing Crates
Each crate is publishing-ready with comprehensive metadata:
```bash
# Validate before publishing
cargo publish --dry-run -p tiles_tools

# Publish to crates.io
cargo publish -p tiles_tools
```

### Adding New Features
1. Create your module in the appropriate `module/` subdirectory
2. Add workspace dependency in root `Cargo.toml` 
3. Follow the established patterns for web integration
4. Add comprehensive examples and documentation

## 📚 Documentation

- **[API Documentation](https://docs.rs/)** - Complete API reference for all crates
- **[Interactive Examples](./examples/)** - 30+ WebGL/WebGPU demos and tutorials  
- **[Crate Documentation](https://docs.rs/tiles_tools)** - Individual crate documentation
- **[WebAssembly Guide](./examples/readme.md)** - Web deployment and optimization

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](./CONTRIBUTING.md) for details.

### Key Areas for Contribution
- 🎨 **Graphics Algorithms** - New rendering techniques and optimizations
- 🎮 **Game Systems** - ECS components, AI, physics integration  
- 📱 **Platform Support** - Mobile web, progressive web app features
- 📖 **Documentation** - Examples, tutorials, and API improvements

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](./LICENSE) file for details.

## 🌟 Showcase

*Built something amazing with CGTools? [Submit your project](https://github.com/Wandalen/cgtools/discussions) to be featured here!*

---

<div align="center">

**[🌐 View Live Examples](https://wandalen.github.io/cgtools)**  
**[📚 Read the Docs](https://docs.rs/cgtools)**  
**[💬 Join Discussions](https://github.com/Wandalen/cgtools/discussions)**

Made with ❤️ by the CGTools team

</div>
