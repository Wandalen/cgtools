# 🎨 CGTools - Web-First Computer Graphics Toolkit

> **Modern, performant computer graphics tools built for the web platform**

CGTools is a comprehensive Rust-based toolkit for computer graphics programming, specifically designed for WebAssembly and web deployment. It provides everything you need to build interactive graphics applications, games, and visualizations that run natively in the browser.

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
- Rust 1.70+ with WebAssembly target: `rustup target add wasm32-unknown-unknown`
- Web server for serving examples (Python: `python -m http.server`)

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

## 📦 Core Modules

| Module | Description | Use Cases |
|--------|-------------|-----------|
| **`minwebgl`** | WebGL 2.0 wrapper and utilities | 3D rendering, shaders, textures |
| **`minwebgpu`** | WebGPU bindings and abstractions | Modern compute and graphics |
| **`tiles_tools`** | Tile-based game engine components | Grid games, pathfinding, FOV |
| **`ndarray_cg`** | N-dimensional arrays for graphics | Matrix math, transformations |
| **`browser_input`** | Web input event handling | Mouse, keyboard, touch input |
| **`renderer`** | High-level rendering abstractions | Scene graphs, lighting |
| **`vectorizer`** | Vector graphics tools | SVG generation, path tracing |

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

# Build for web (specific example)
cd examples/minwebgl/trivial
wasm-pack build --target web

# Run with optimizations
cargo build --release --target wasm32-unknown-unknown
```

### Adding New Features
1. Create your module in the appropriate `module/` subdirectory
2. Add workspace dependency in root `Cargo.toml` 
3. Follow the established patterns for web integration
4. Add comprehensive examples and documentation

## 📚 Documentation

- **[API Documentation](https://docs.rs/cgtools)** - Complete API reference
- **[Examples](./examples/)** - Interactive demos and tutorials  
- **[Architecture Guide](./docs/architecture.md)** - Design principles and patterns
- **[WebAssembly Integration](./docs/wasm.md)** - Web deployment guide

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
