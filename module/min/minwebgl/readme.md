# 🎮 minwebgl

> **Minimal, type-safe WebGL 2.0 wrapper for Rust and WebAssembly**

A concise and performant WebGL 2.0 abstraction layer designed specifically for Rust applications targeting WebAssembly. Built with ergonomics and safety in mind, minwebgl eliminates WebGL boilerplate while maintaining full control over rendering pipelines.

## ✨ Features

### 🚀 **Core Capabilities**
- **Modern WebGL 2.0** - Full WebGL 2.0 API coverage with type safety
- **Zero-Cost Abstractions** - Minimal overhead over raw WebGL calls
- **Memory Safe** - Rust ownership prevents common WebGL errors
- **WebAssembly Optimized** - Designed for efficient WASM deployment

### 🛠️ **Rendering Features**
- **Attribute Management** - Type-safe vertex attribute uploading
- **Matrix Support** - Row-major matrix handling in attributes
- **Instanced Rendering** - Efficient batch rendering support
- **Uniform Buffer Objects** - Modern uniform data management
- **Vertex Array Objects** - Optimized vertex state caching
- **Shader Management** - Compile-time shader validation
- **Texture Operations** - 2D/3D textures, cube maps, and arrays
- **Framebuffer Control** - Render-to-texture and multi-target rendering

## 🚀 Quick Start

### Add to Your Project
```toml
[dependencies]
minwebgl = { workspace = true, features = ["enabled"] }
wasm-bindgen = "0.2"
web-sys = "0.3"
```

### Basic Triangle Example
```rust,ignore
use minwebgl as gl;
use wasm_bindgen::prelude::*;

#[ wasm_bindgen( start ) ]
pub fn main() -> Result< (), JsValue >
{
  // Create canvas and WebGL context
  let canvas = gl::canvas::make()?;
  let gl_context = gl::context::from_canvas( &canvas )?;
  
  // Vertex shader
  let vertex_src = r#"
    attribute vec2 position;
    void main() {
      gl_Position = vec4(position, 0.0, 1.0);
    }
  "#;
  
  // Fragment shader
  let fragment_src = r#"
    precision mediump float;
    void main() {
      gl_FragColor = vec4(1.0, 0.0, 0.0, 1.0);
    }
  "#;
  
  // Create shader program
  let program = gl::shader::make(&gl_context, vertex_src, fragment_src)?;
  
  // Triangle vertices
  let vertices: [f32; 6] = [
    0.0,  0.5,  // Top
     -0.5, -0.5,  // Bottom left
    0.5, -0.5,  // Bottom right
  ];
  
  // Create and bind vertex buffer
  let buffer = gl::buffer::make(&gl_context);
  gl::buffer::bind(&gl_context, &buffer);
  gl::buffer::upload_f32(&gl_context, &vertices);
  
  // Set up vertex attributes
  let pos_attrib = gl::program::get_attrib_location(&gl_context, &program, "position");
  gl::attrib::enable_vertex_attrib_array(&gl_context, pos_attrib);
  gl::attrib::vertex_attrib_pointer_f32(&gl_context, pos_attrib, 2, false, 0, 0);
  
  // Draw triangle
  gl::program::use_program(&gl_context, &program);
  gl::draw::arrays(&gl_context, gl::DrawMode::Triangles, 0, 3);
  
  Ok(())
}
```

### Advanced Features Example
```rust,ignore
use minwebgl as gl;

// Instanced rendering with matrices
let instances = vec![
  gl::Mat4::identity(),
  gl::Mat4::translation([1.0, 0.0, 0.0]),
  gl::Mat4::translation([-1.0, 0.0, 0.0]),
];

// Upload instance matrices
let instance_buffer = gl::buffer::make(&gl_context);
gl::buffer::bind(&gl_context, &instance_buffer);
gl::buffer::upload_matrix_4x4(&gl_context, &instances);

// Set up instanced attribute (divisor = 1)
gl::attrib::vertex_attrib_divisor(&gl_context, matrix_attrib, 1);

// Instanced draw call
gl::draw::arrays_instanced(&gl_context, gl::DrawMode::Triangles, 0, 3, instances.len());
```

## 🛠️ Building and Deployment

### Prerequisites
```bash
rustup target add wasm32-unknown-unknown
```

### Option 1: wasm-pack (Recommended)
```bash
# Install wasm-pack
cargo install wasm-pack

# Build for web
wasm-pack build --target web --out-dir pkg

# Use in HTML
```html
<script type="module">
  import init from "./pkg/your_crate_name.js";
  init();
</script>
```

### Option 2: Trunk (Development)
```bash
# Install trunk
cargo install trunk

# Serve with hot reload
trunk serve --release

# Build for production
trunk build --release
```

For asset loading with Trunk:
```html
<link data-trunk rel="copy-dir" href="assets/" data-target-path="static"/>
```

## 📚 API Overview

| Module | Description | Key Functions |
|--------|-------------|---------------|
| `canvas` | Canvas creation and management | `make()`, `resize()` |
| `context` | WebGL context initialization | `from_canvas()`, `from_canvas_with()` |
| `shader` | Shader compilation and programs | `make()`, `compile_vertex()`, `compile_fragment()` |
| `buffer` | Buffer operations | `make()`, `upload_f32()`, `upload_matrix_4x4()` |
| `attrib` | Vertex attributes | `vertex_attrib_pointer_f32()`, `enable_vertex_attrib_array()` |
| `texture` | Texture management | `make_2d()`, `upload_2d()`, `bind()` |
| `framebuffer` | Render targets | `make()`, `bind()`, `attach_texture()` |
| `draw` | Draw commands | `arrays()`, `elements()`, `arrays_instanced()` |

## 🎯 Examples

- **[Basic Triangle](../../../examples/minwebgl/minimal/)** - Simple triangle rendering
- **[Hexagonal Grid](../../../examples/minwebgl/hexagonal_grid/)** - Interactive grid with pathfinding
- **[Deferred Shading](../../../examples/minwebgl/deferred_shading/)** - Advanced lighting pipeline
- **[Text Rendering](../../../examples/minwebgl/text_msdf/)** - GPU text rendering with MSDF
- **[Object Picking](../../../examples/minwebgl/object_picking/)** - Mouse interaction with 3D objects

## 🔧 Advanced Usage

### Custom Context Options
```rust,ignore
let options = gl::context::ContextOptions::default()
  .antialias(false)
  .alpha(true)
  .depth(true)
  .stencil(false)
  .premultiplied_alpha(false);

let gl_context = gl::context::from_canvas_with(&canvas, options)?;
```

### Uniform Buffer Objects
```rust,ignore
// Create UBO
let ubo = gl::buffer::make(&gl_context);
gl::buffer::bind_uniform(&gl_context, &ubo);

// Upload uniform data
let uniforms = MyUniformStruct { ... };
gl::buffer::upload_uniform(&gl_context, &uniforms);

// Bind to shader
let block_index = gl::program::get_uniform_block_index(&gl_context, &program, "UniformBlock");
gl::program::uniform_block_binding(&gl_context, &program, block_index, 0);
gl::buffer::bind_buffer_base_uniform(&gl_context, 0, &ubo);
```

### Error Handling
```rust,ignore
// Enable debug output
gl::debug::enable(&gl_context);

// Check for errors
if let Some(error) = gl::get_error(&gl_context) {
  eprintln!("WebGL error: {:?}", error);
}
```

## 🤝 Contributing

This crate is part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT