# ⚡ minwebgpu

> **Modern WebGPU wrapper for next-generation web graphics**

A safe, ergonomic Rust wrapper around WebGPU for high-performance graphics and compute in the browser. Built for the future of web graphics with compute shader support, modern rendering pipelines, and optimal WebAssembly integration.

## ✨ Features

### 🚀 **Next-Gen Graphics**
- **WebGPU Native** - Direct WebGPU API access with Rust safety
- **Compute Shaders** - GPU compute for parallel processing
- **Modern Pipelines** - Descriptor-based render and compute pipelines
- **Memory Efficient** - Optimal memory management and buffer operations

### 🛠️ **Core Capabilities**
- **Render Pipelines** - Advanced rendering with modern GPU features
- **Compute Pipelines** - Parallel computation on the GPU
- **Buffer Management** - Type-safe buffer operations and memory handling
- **Texture Support** - 2D/3D textures, cube maps, and texture arrays
- **Shader Modules** - WGSL shader compilation and validation
- **Command Encoding** - Efficient command buffer recording

## 🚀 Quick Start

### Add to Your Project
```toml
[dependencies]
minwebgpu = { workspace = true, features = ["enabled"] }
wasm-bindgen = "0.2"
web-sys = "0.3"
```

### Basic Setup
```rust
use minwebgpu as gpu;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub async fn main() -> Result<(), JsValue> {
  // Get WebGPU adapter and device
  let instance = gpu::Instance::new()?;
  let adapter = instance.request_adapter().await?;
  let device = adapter.request_device().await?;
  
  // Create canvas and surface
  let canvas = gpu::canvas::make()?;
  let surface = instance.create_surface(&canvas)?;
  
  Ok(())
}
```

### Simple Triangle Render Pipeline
```rust
use minwebgpu as gpu;

// WGSL vertex shader
let vertex_shader = r#"
@vertex
fn vs_main(@location(0) position: vec2<f32>) -> @builtin(position) vec4<f32> {
    return vec4<f32>(position, 0.0, 1.0);
}
"#;

// WGSL fragment shader  
let fragment_shader = r#"
@fragment
fn fs_main() -> @location(0) vec4<f32> {
    return vec4<f32>(1.0, 0.0, 0.0, 1.0);
}
"#;

// Create shader modules
let vs_module = gpu::shader::create(&device, vertex_shader);
let fs_module = gpu::shader::create(&device, fragment_shader);

// Create render pipeline
let pipeline = gpu::render_pipeline::desc(
    gpu::VertexState::new(&vs_module)
        .buffer(&vertex_layout)
)
.fragment(
    gpu::FragmentState::new(&fs_module)
        .target(gpu::ColorTargetState::new().format(surface_format))
)
.primitive(gpu::PrimitiveState::new().triangle_list())
.create(&device)?;
```

### Compute Shader Example
```rust
use minwebgpu as gpu;

// WGSL compute shader
let compute_shader = r#"
@group(0) @binding(0) var<storage, read_write> data: array<f32>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let index = global_id.x;
    if (index >= arrayLength(&data)) { return; }
    
    data[index] = data[index] * 2.0;
}
"#;

// Create compute pipeline
let compute_module = gpu::shader::create(&device, compute_shader);
let compute_pipeline = gpu::compute_pipeline::desc(
  gpu::ComputeState::new(&compute_module)
).create(&device)?;

// Create storage buffer
let buffer = gpu::BufferInitDescriptor::new(
    &input_data,
    gpu::BufferUsage::STORAGE | gpu::BufferUsage::COPY_SRC
).create(&device)?;

// Dispatch compute work
let mut encoder = device.create_command_encoder();
let mut compute_pass = encoder.begin_compute_pass();
compute_pass.set_pipeline(&compute_pipeline);
compute_pass.set_bind_group(0, &bind_group);
compute_pass.dispatch_workgroups(workgroup_count, 1, 1);
compute_pass.end();
```

## 📚 API Overview

| Module | Description | Key Types |
|--------|-------------|-----------|
| `instance` | WebGPU instance and adapter | `Instance`, `Adapter` |
| `device` | Device and queue management | `Device`, `Queue` |
| `shader` | WGSL shader compilation | `ShaderModule`, `create()` |
| `buffer` | Buffer operations | `Buffer`, `BufferInitDescriptor` |
| `texture` | Texture management | `Texture`, `TextureView`, `Sampler` |
| `render_pipeline` | Render pipeline creation | `RenderPipeline`, `desc()` |
| `compute_pipeline` | Compute pipeline creation | `ComputePipeline`, `desc()` |
| `command` | Command encoding | `CommandEncoder`, `RenderPass`, `ComputePass` |

## 🎯 Key Concepts

### Render Pipelines
WebGPU uses descriptor-based pipeline creation:

```rust
let pipeline = gpu::render_pipeline::desc(
  gpu::VertexState::new(&vs_module)
    .buffer(&vertex_buffer_layout)
)
.fragment(
  gpu::FragmentState::new(&fs_module)
    .target(gpu::ColorTargetState::new().format(gpu::TextureFormat::Bgra8unormSrgb))
)
.primitive(gpu::PrimitiveState::new().triangle_list())
.depth_stencil(gpu::DepthStencilState::new())
.multisample(gpu::MultisampleState::new().count(4))
.create(&device)?;
```

### Buffer Management
Type-safe buffer operations:

```rust
// Vertex buffer
let vertices: [Vertex; 3] = [...];
let vertex_buffer = gpu::BufferInitDescriptor::new(
  &vertices,
  gpu::BufferUsage::VERTEX
).create(&device)?;

// Uniform buffer
let uniforms = UniformData { ... };
let uniform_buffer = gpu::BufferInitDescriptor::new(
  &uniforms,
  gpu::BufferUsage::UNIFORM | gpu::BufferUsage::COPY_DST
).create(&device)?;
```

### Bind Groups
Resource binding for shaders:

```rust
let bind_group_layout = device.create_bind_group_layout(&gpu::BindGroupLayoutDescriptor {
  entries: &[
    gpu::BindGroupLayoutEntry::uniform(0, gpu::ShaderStage::VERTEX),
    gpu::BindGroupLayoutEntry::texture(1, gpu::ShaderStage::FRAGMENT),
    gpu::BindGroupLayoutEntry::sampler(2, gpu::ShaderStage::FRAGMENT),
  ],
});

let bind_group = device.create_bind_group(&gpu::BindGroupDescriptor {
  layout: &bind_group_layout,
  entries: &[
    gpu::BindGroupEntry::buffer(0, &uniform_buffer),
    gpu::BindGroupEntry::texture_view(1, &texture_view),
    gpu::BindGroupEntry::sampler(2, &sampler),
  ],
});
```

## 🎮 Examples

- **[Hello Triangle](../../../examples/minwebgpu/hello_triangle/)** - Basic WebGPU triangle
- **[Deferred Rendering](../../../examples/minwebgpu/deffered_rendering/)** - Advanced lighting with WebGPU
- **[Compute Particles](../../../examples/minwebgpu/compute_particles/)** - GPU particle simulation

## 🔧 Advanced Features

### Compute Shaders
```rust
// Parallel array processing
let compute_pipeline = gpu::compute_pipeline::desc(
  gpu::ComputeState::new(&compute_module)
).create(&device)?;

// Dispatch with workgroups
compute_pass.dispatch_workgroups(
  (data_size + 63) / 64, // Round up to workgroup size
  1,
  1
);
```

### Multi-Target Rendering
```rust
let pipeline = gpu::render_pipeline::desc(vertex_state)
.fragment(
  gpu::FragmentState::new(&fs_module)
    .target(gpu::ColorTargetState::new().format(gpu::TextureFormat::Rgba8unorm))
    .target(gpu::ColorTargetState::new().format(gpu::TextureFormat::Rgba16Float))
)
.create(&device)?;
```

### Memory-Mapped Buffers
```rust
// Create mappable buffer
let buffer = device.create_buffer(&gpu::BufferDescriptor {
  size: data.len() as u64,
  usage: gpu::BufferUsage::MAP_READ | gpu::BufferUsage::COPY_DST,
  mapped_at_creation: false,
});

// Map and read data
let buffer_slice = buffer.slice(..);
buffer_slice.map_async(gpu::MapMode::Read).await?;
let data = buffer_slice.get_mapped_range();
// Process data...
buffer.unmap();
```

## 🛠️ Building

WebGPU requires modern browser support:

### Browser Requirements
- **Chrome/Edge**: Version 94+
- **Firefox**: Version 110+ (behind flag)
- **Safari**: Version 16.4+

### Build Commands
```bash
# Build for web
wasm-pack build --target web --out-dir pkg

# Development with trunk
trunk serve --release
```

