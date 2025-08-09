# 🚀 CGTools Examples

Interactive examples showcasing computer graphics techniques and WebAssembly rendering with CGTools. Each example demonstrates specific concepts and real-world applications.

## 📂 Example Categories

### 🎮 **MinWebGL Examples**
Interactive WebGL 2.0 examples running in browsers:

| Category | Examples | Description |
|----------|----------|-------------|
| **Core Rendering** | hexagonal_grid, deferred_shading | Fundamental rendering techniques |
| **Visual Effects** | filters, outline, narrow_outline | Post-processing and visual enhancements |
| **3D Graphics** | gltf_viewer, obj_viewer, make_cube_map | 3D model loading and environment mapping |
| **Advanced Techniques** | raycaster, sprite_animation, text_msdf | Specialized rendering methods |
| **Optimization** | minimize_wasm, spinning_cube_size_opt | Performance and size optimization |
| **Procedural** | wave_function_collapse, mapgen_tiles_rendering | Procedural generation techniques |

### 🖥️ **MinWebGPU Examples** 
Next-generation graphics with WebGPU:

| Example | Focus | Key Features |
|---------|--------|--------------|
| **hello_triangle** | Basics | Simple triangle rendering with WebGPU |
| **deferred_rendering** | Advanced | Multi-pass rendering pipeline |

### 📊 **Math Examples**
Mathematical concepts and algorithms:

| Example | Topic | Implementation |
|---------|--------|----------------|
| **life** | Cellular Automata | Conway's Game of Life simulation |

## 🚀 Quick Start

### Prerequisites
- Rust 1.75+ with `rustup target add wasm32-unknown-unknown`
- Trunk: `cargo install trunk`
- Modern web browser with WebGL 2.0/WebGPU support

### Running Examples

```bash
# Navigate to any example
cd minwebgl/hexagonal_grid

# Run the example
trunk serve --release

# Open http://localhost:8080 in your browser
```

### Development & Building
```bash
# Development mode (faster compilation)
trunk serve

# Build for production
trunk build --release
```

## 🎯 Featured Examples

### 🔥 **Must-See Demonstrations**

#### **Deferred Shading** (`minwebgl/deferred_shading`)
Advanced multi-pass rendering technique for handling many lights efficiently.
- G-buffer generation and lighting passes
- HDR tone mapping and post-processing
- Real-time performance with multiple light sources

#### **glTF Viewer** (`minwebgl/gltf_viewer`)
Complete 3D model viewer supporting the glTF 2.0 standard.
- PBR material rendering
- Animation playback
- Interactive camera controls
- KHR extension support

#### **Wave Function Collapse** (`minwebgl/wave_function_collapse`)
Procedural generation using constraint-solving algorithms.
- Tile-based pattern generation
- Real-time constraint solving
- Interactive parameter adjustment

#### **Hexagonal Grid** (`minwebgl/hexagonal_grid`)
Comprehensive hexagonal coordinate system demonstration.
- Multiple coordinate representations (axial, cube, offset)
- Pathfinding algorithms
- Interactive grid manipulation

## 🛠️ Example Structure

Each example follows a consistent structure:

```
example_name/
├── src/
│   ├── main.rs          # Application entry point
│   ├── lib.rs           # Core logic
│   └── ...              # Additional modules
├── Cargo.toml           # Rust dependencies
├── index.html           # HTML template
├── readme.md            # Example documentation
└── assets/              # Static assets (if needed)
    ├── models/
    ├── textures/
    └── shaders/
```

## 📖 Learning Path

### Beginner (Start Here)
1. **hello_triangle** (minwebgpu) - Basic rendering concepts
2. **hexagonal_grid** (minwebgl) - Coordinate systems and input handling
3. **spinning_cube_size_opt** (minwebgl) - 3D transformations and optimization

### Intermediate
1. **gltf_viewer** (minwebgl) - 3D asset loading and PBR materials
2. **deferred_shading** (minwebgl) - Advanced rendering pipelines
3. **raycaster** (minwebgl) - Ray-based rendering techniques

### Advanced
1. **wave_function_collapse** (minwebgl) - Procedural generation algorithms
2. **text_msdf** (minwebgl) - Advanced text rendering
3. **minimize_wasm** (minwebgl) - WebAssembly optimization techniques

## 🔧 Development Tips

- **Performance**: Use `trunk serve --release` for testing performance
- **Debugging**: Use `trunk serve` for better error messages
- **Assets**: Place files in `assets/` directory - Trunk copies them automatically
- **Shaders**: Embed GLSL directly in Rust or load as separate files

## 🌐 Browser Compatibility

| Feature | Chrome | Firefox | Safari | Edge |
|---------|--------|---------|--------|------|
| WebGL 2.0 | ✅ | ✅ | ✅ | ✅ |
| WebGPU | ✅ | 🚧 | 🚧 | ✅ |
| WASM SIMD | ✅ | ✅ | ✅ | ✅ |
| SharedArrayBuffer | ⚠️ | ⚠️ | ⚠️ | ⚠️ |

**Legend:** ✅ Full Support | 🚧 Experimental | ⚠️ Requires Flags

## 🤝 Contributing Examples

When adding new examples:
1. Follow the standard structure shown above
2. Include comprehensive README with implementation details  
3. Add inline documentation explaining key concepts
4. Test across browsers for compatibility

## 📚 Additional Resources

- **CGTools Documentation** - Core library documentation
- **WebGL Reference** - Graphics API documentation  
- **WebGPU Specification** - Next-generation graphics API
- **Computer Graphics Theory** - Mathematical foundations
- **Performance Best Practices** - Optimization techniques

## 🐛 Troubleshooting

**Build Errors:**
```bash
trunk clean && cargo clean && trunk build --release
```

**Runtime Issues:**
- Check browser console for errors
- Verify WebGL/WebGPU support  
- Use `trunk serve --release` for performance testing

**Getting Help:**
- Check example README files
- Review CGTools documentation
- Browse source code for implementation details
