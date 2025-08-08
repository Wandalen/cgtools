# 🚀 CGTools Examples

> **Comprehensive examples showcasing computer graphics techniques and WebAssembly rendering**

This directory contains a rich collection of examples demonstrating the capabilities of CGTools across different graphics APIs, mathematical concepts, and rendering techniques. Each example is designed to be educational, showcasing best practices and real-world applications.

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
- [Rust](https://rustup.rs/) (latest stable)
- [Trunk](https://trunkrs.dev/) for WebAssembly builds
- Modern web browser with WebGL 2.0/WebGPU support

### Running Examples

1. **Install Trunk** (if not already installed):
   ```bash
   cargo install trunk
   ```

2. **Navigate to any example**:
   ```bash
   cd minwebgl/hexagonal_grid
   ```

3. **Run the example**:
   ```bash
   trunk serve --release
   ```

4. **Open your browser** to `http://localhost:8080`

### Development Mode
For faster compilation during development:
```bash
trunk serve --dev
```

### Building for Production
To build optimized WebAssembly bundles:
```bash
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

### Performance Profiling
Use browser DevTools to profile your applications:
```bash
trunk serve --release --features="profile"
```

### Debugging WebAssembly
Enable debug symbols for better error messages:
```bash
trunk serve --dev --features="debug"
```

### Asset Management
Place assets in the `assets/` directory - Trunk will automatically copy them to the build output.

### Custom Shaders
GLSL shaders can be embedded directly in Rust code or loaded as separate files during build.

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

1. **Follow the standard structure** shown above
2. **Include comprehensive README** with theory and implementation details
3. **Add inline documentation** explaining key concepts
4. **Optimize for learning** - prioritize clarity over brevity
5. **Test across browsers** to ensure compatibility

### Example Template
Use this template when creating new examples:
```bash
# Copy template structure
cp -r _template your_example_name
cd your_example_name
# Edit Cargo.toml, src/, and README.md
```

## 📚 Additional Resources

- **CGTools Documentation** - Core library documentation
- **WebGL Reference** - Graphics API documentation  
- **WebGPU Specification** - Next-generation graphics API
- **Computer Graphics Theory** - Mathematical foundations
- **Performance Best Practices** - Optimization techniques

## 🐛 Troubleshooting

### Common Issues

**Build Errors:**
```bash
# Clean and rebuild
trunk clean
cargo clean
trunk build --release
```

**Runtime Errors:**
- Check browser console for JavaScript errors
- Verify WebGL/WebGPU support in your browser
- Ensure WASM is enabled

**Performance Issues:**
- Use `--release` flag for production builds
- Profile with browser DevTools
- Consider reducing complexity for mobile devices

### Getting Help
- Check example-specific README files for detailed guidance
- Review the main CGTools documentation
- Browse the source code for implementation details
