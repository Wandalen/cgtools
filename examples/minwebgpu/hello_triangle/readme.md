# ▲ WebGPU Hello Triangle

> **Your first WebGPU triangle - the classic "Hello World" of graphics programming**

A minimal but complete WebGPU example that renders a colorful triangle to the screen. This is the foundational example for learning WebGPU concepts including render pipelines, shaders, and GPU command encoding.

![WebGPU Triangle](./showcase.jpg)

## ✨ What You'll Learn

### 🎯 **Core WebGPU Concepts**
- **WebGPU Instance** - Creating and configuring the graphics API
- **Adapter & Device** - Selecting and initializing GPU hardware
- **Surface Configuration** - Setting up canvas rendering target
- **Render Pipelines** - Modern graphics pipeline creation

### 🛠️ **Essential Components**
- **WGSL Shaders** - WebGPU's shader language basics
- **Vertex Buffers** - Uploading triangle geometry to GPU
- **Command Encoding** - Recording GPU commands for execution
- **Frame Rendering** - Complete render loop implementation

## 🚀 Quick Start

### Prerequisites
- Modern browser with WebGPU support (Chrome 94+, Firefox 110+, Safari 16.4+)
- Rust with `wasm32-unknown-unknown` target
- Trunk or wasm-pack for building

### Run the Example
```bash
# Navigate to hello triangle example
cd examples/minwebgpu/hello_triangle

# Option 1: Using trunk (recommended for development)
trunk serve --release

# Option 2: Using wasm-pack
wasm-pack build --target web --out-dir pkg
# Then serve the directory with any static server
```

Open http://localhost:8080 to see your WebGPU triangle!

## 🔧 Code Breakdown

### Vertex Data
```rust
// Triangle vertices in normalized device coordinates
let vertices = [
    // Position     Color
    0.0,  0.8,     1.0, 0.0, 0.0,  // Top (red)
   -0.7, -0.8,     0.0, 1.0, 0.0,  // Bottom left (green)  
    0.7, -0.8,     0.0, 0.0, 1.0,  // Bottom right (blue)
];
```

### WGSL Vertex Shader
```wgsl
@vertex
fn vs_main(@location(0) position: vec2<f32>, @location(1) color: vec3<f32>) 
    -> @builtin(position) vec4<f32> 
{
  return vec4<f32>(position, 0.0, 1.0);
}
```

### WGSL Fragment Shader
```wgsl
@fragment  
fn fs_main() -> @location(0) vec4<f32> 
{
  return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Red triangle
}
```

### Render Pipeline Setup
```rust
let render_pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
  vertex: VertexState {
    module: &shader_module,
    entry_point: "vs_main",
    buffers: &[vertex_buffer_layout],
  },
  fragment: Some(FragmentState {
    module: &shader_module,
    entry_point: "fs_main", 
    targets: &[ColorTargetState {
      format: surface_format,
      blend: None,
      write_mask: ColorWrites::ALL,
    }],
  }),
  primitive: PrimitiveState::default(),
  // ... other settings
});
```

## 🎯 Key Learning Points

### WebGPU vs WebGL Differences
- **Explicit Resource Management** - Manual buffer and texture lifecycle
- **Command-Based Rendering** - Record commands, then submit batches
- **Modern API Design** - Descriptor-based object creation
- **Compute Integration** - Unified graphics and compute pipelines

### Coordinate Systems
- **NDC Space** - Normalized Device Coordinates (-1 to +1)
- **Y-Up Convention** - Positive Y points upward in WebGPU
- **Z-Range** - Depth values from 0.0 (near) to 1.0 (far)

### Pipeline Stages
1. **Vertex Stage** - Process vertex attributes and positions
2. **Primitive Assembly** - Connect vertices into triangles
3. **Rasterization** - Generate fragments for triangle coverage
4. **Fragment Stage** - Compute final pixel colors

## 🔗 Next Steps

### Try These Modifications
```rust
// 1. Change triangle colors by modifying vertex data
let vertices = [
    0.0,  0.8,     1.0, 1.0, 0.0,  // Yellow top
   -0.7, -0.8,     1.0, 0.0, 1.0,  // Magenta left
    0.7, -0.8,     0.0, 1.0, 1.0,  // Cyan right
];

// 2. Add vertex color interpolation in shaders
// Pass color from vertex to fragment shader

// 3. Create multiple triangles
let triangle_count = 3;
render_pass.draw(0..3 * triangle_count, 0..1);
```

### Advanced Examples
- **[Deferred Rendering](../deffered_rendering/)** - Multi-pass rendering with G-buffers
- **[Compute Shaders](../compute_particles/)** - GPU parallel processing
- **[Texture Loading](../textured_quad/)** - Image rendering with samplers

## 📚 Resources

- **[WebGPU Specification](https://gpuweb.github.io/gpuweb/)** - Official WebGPU standard
- **[WGSL Specification](https://gpuweb.github.io/gpuweb/wgsl/)** - WebGPU shader language
- **[WebGPU Samples](https://webgpu.github.io/webgpu-samples/)** - Official examples collection
- **[Learn WebGPU](https://eliemichel.github.io/LearnWebGPU/)** - Comprehensive tutorial series

## 🛠️ Troubleshooting

### Browser Compatibility
```javascript
// Check WebGPU support
if (!navigator.gpu) {
  console.error("WebGPU not supported");
}
```

### Common Issues
- **Shader Compilation Errors** - Check WGSL syntax and entry points
- **Buffer Size Mismatches** - Ensure vertex data matches buffer layout
- **Surface Configuration** - Verify canvas format matches pipeline

### Debug Tools
- **Browser DevTools** - WebGPU error messages in console
- **Shader Validation** - Built-in WGSL syntax checking
- **Performance Profiler** - GPU timing and resource usage

## 🤝 Contributing

Part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT