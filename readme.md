# 🎨 CGTools - Web-First Computer Graphics Toolkit

[![CI](https://github.com/Wandalen/cgtools/workflows/CI/badge.svg)](https://github.com/Wandalen/cgtools/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)
[![WASM](https://img.shields.io/badge/WebAssembly-Ready-brightgreen)](https://webassembly.org/)

> **Modern computer graphics toolkit built for WebAssembly and the web**

Comprehensive Rust-based toolkit for interactive graphics, games, and visualizations running in browsers with near-native performance.

![Abstract Art](./assets/primitives.jpg)

## ✨ Features

- **WebGL 2.0 & WebGPU** - Modern graphics APIs with zero-copy operations
- **Advanced Mathematics** - N-dimensional arrays, linear algebra, geometric primitives
- **Game Development** - Tile systems, pathfinding, ECS, input handling
- **Specialized Tools** - 3D models, embroidery patterns, vector graphics

## 🚀 Quick Start

### Prerequisites
- **Rust 1.75+** with WebAssembly target: `rustup target add wasm32-unknown-unknown`
- **Trunk**: `cargo install trunk`

### Try an Example
```bash
# Clone and navigate to project
git clone https://github.com/Wandalen/cgtools
cd cgtools

# Run a WebGL example
cd examples/minwebgl/hexagonal_grid
trunk serve --release
# Open http://localhost:8080 in your browser
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
# Test all crates
cargo test --workspace

# Check compilation
cargo check --workspace

# Run examples with Trunk
cd examples/minwebgl/trivial
trunk serve --release
```

## 📚 Documentation

- **[API Documentation](https://docs.rs/)** - Complete API reference for all crates
- **[Interactive Examples](./examples/)** - 30+ WebGL/WebGPU demos and tutorials  
- **[Crate Documentation](https://docs.rs/tiles_tools)** - Individual crate documentation
- **[WebAssembly Guide](./examples/readme.md)** - Web deployment and optimization


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
