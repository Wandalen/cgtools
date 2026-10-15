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
- **Texture Operations** - 2D textures and cube maps (`texture::d2`, `texture::cube`)
- **Framebuffer Control** - Render-to-texture and multi-target rendering

## 🚀 Quick Start

### Add to Your Project
```toml
[dependencies]
minwebgl = { workspace = true }
```

`enabled` is already part of minwebgl's own `default` feature set (see `Cargo.toml`'s `[features]` section) — no separate `features = [...]` override, `wasm-bindgen`, or `web-sys` version pin is needed unless your own crate calls those APIs directly. Every real example under `examples/minwebgl/` depends on `minwebgl` alone.

### Basic Triangle Example

Lifted verbatim from
[`examples/minwebgl/context_triangle_smoke/src/main.rs`](../../../examples/minwebgl/context_triangle_smoke/src/main.rs)
— the browser pixel-verified smoke test for this exact call sequence (`browsee`-driven; see that crate's own `tests/manual/readme.md`):

```rust,ignore
use minwebgl as gl;
use gl::GL;

fn app_run() -> Result< (), gl::WebglError >
{
  gl::browser::setup( gl::browser::Config::default() );
  let canvas = gl::canvas::make()?;
  let context = gl::context::from_canvas( &canvas )?;

  let vertex_shader_src = include_str!( "../shaders/shader.vert" );
  let fragment_shader_src = include_str!( "../shaders/shader.frag" );
  let program = gl::Program::new( context.clone(), vertex_shader_src, fragment_shader_src )?;
  program.activate();

  let vertex_data : [ f32 ; 6 ] = [ -0.5, -0.5, 0.5, -0.5, 0.0, 0.5 ];
  let vertex_buffer = gl::buffer::create( &context )?;
  gl::buffer::upload( &context, &vertex_buffer, &vertex_data, GL::STATIC_DRAW );

  let vao = gl::vao::create( &context )?;
  context.bind_vertex_array( Some( &vao ) );
  gl::BufferDescriptor::new::< [ f32 ; 2 ] >().stride( 2 ).offset( 0 ).attribute_pointer( &context, 0, &vertex_buffer )?;

  context.clear_color( 0.0, 0.0, 0.0, 1.0 );
  context.clear( GL::COLOR_BUFFER_BIT );
  context.draw_arrays( GL::TRIANGLES, 0, 3 );
  context.bind_vertex_array( None );

  Ok( () )
}

fn main()
{
  app_run().unwrap();
}
```

`shader.vert`/`shader.frag` are real WebGL 2.0 GLSL ES 3.00 (`#version 300 es`, `in`/`out`) — minwebgl targets WebGL 2.0 exclusively, so ES 1.00-style `attribute`/`gl_FragColor` shaders (as an older revision of this example showed) will not compile against it.

### Advanced Features Example

Instanced rendering, condensed from
[`examples/minwebgl/attributes_instanced/src/main.rs`](../../../examples/minwebgl/attributes_instanced/src/main.rs)
(full listing there includes the buffer literal data): `BufferDescriptor::divisor(N)` controls per-attribute update frequency — `0` = per-vertex, `1` = per-instance, `2+` = shared across that many instances — and the draw call is `WebGl2RenderingContext::draw_arrays_instanced` directly (no minwebgl wrapper):

```rust,ignore
use minwebgl as gl;
use gl::GL;

let position_slot = 0;
let position_buffer = gl::buffer::create( &gl )?;
gl::buffer::upload( &gl, &position_buffer, &position_data, GL::STATIC_DRAW );

let offset_slot = 2;
let offset_buffer = gl::buffer::create( &gl )?;
gl::buffer::upload( &gl, &offset_buffer, &offset_data, GL::STATIC_DRAW );

let vao = gl::vao::create( &gl )?;
gl.bind_vertex_array( Some( &vao ) );
gl::BufferDescriptor::new::< [ f32 ; 2 ] >().stride( 2 ).offset( 0 ).divisor( 0 )
  .attribute_pointer( &gl, position_slot, &position_buffer )?;
gl::BufferDescriptor::new::< [ f32 ; 2 ] >().stride( 2 ).offset( 0 ).divisor( 1 )
  .attribute_pointer( &gl, offset_slot, &offset_buffer )?;
gl.bind_vertex_array( None );

gl.bind_vertex_array( Some( &vao ) );
gl.draw_arrays_instanced( GL::TRIANGLES, 0, 3 * 6, 5 ); // 5 instances
gl.bind_vertex_array( None );
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

### Testing

The crate compiles on native targets, so its pure-logic layer is tested natively — no browser needed:

```bash
cargo test -p minwebgl --all-features
```

That invocation runs the `tests/` suite — the public pure-logic API (e.g. `DataType` ↔ `Const` conversions) plus the validation helpers `natoms_validate` and `attachment_id_convert`, extracted for testability during bug fixes and exported at their module paths so their tests live in `tests/` like everything else. Verify yourself: `grep -rn "cfg( test )" src/` lists no inline test modules, `ls tests/` the native suite.

Anything that touches a live GL context or the DOM (context creation, shaders, VAOs, textures, uniforms, file/fetch) is **not** natively testable — but it no longer waits on workspace-level `wasm-bindgen-test` runner infrastructure either. `browsee` (external, Bash-driven real-browser automation, already used elsewhere in this repo) is confirmed sufficient for scripted pixel-verified browser checks: `context::from_canvas` plus a minimal shader/buffer/draw sequence is covered by `examples/minwebgl/context_triangle_smoke/`, driven through the exact command sequence and expected pixel readings documented in `tests/manual/readme.md`. Broader GL-context/DOM coverage (shaders, VAOs, textures, uniforms, file/fetch beyond this one smoke path) remains open — a future task, not a tooling blocker. When building for wasm32 directly, never pass a bare `RUSTFLAGS` value: it clobbers `.cargo/config.toml`'s `--cfg web_sys_unstable_apis`.

## 📚 API Overview

| Module | Description | Key Functions |
|--------|-------------|---------------|
| `canvas` | Canvas creation and management | `make()` (reused from `mingl::web::canvas`) |
| `context` | WebGL context initialization | `from_canvas()`, `from_canvas_with()`, `retrieve_or_make()` |
| `shader` | Shader compilation and linking | `Program::new()`, `ProgramFromSources::new().compile_and_link()` |
| `buffer` | Buffer operations | `create()`, `upload()` |
| `attribute` | Vertex attributes | `BufferDescriptor::new().stride().offset().divisor().attribute_pointer()` |
| `vao` | Vertex array objects | `create()` |
| `texture::d2` / `texture::cube` | Texture upload | `d2::upload()`, `d2::create_and_upload()`, `d2::image_upload_from_path()` |
| `ubo` | Uniform block objects | `upload()`, `diagnostic_info()` (feature `diagnostics`) |
| `drawbuffers` | Multi-render-target attachment lists | `drawbuffers()`, `color_attachment_index_validate()` |
| `clean` | Framebuffer/renderbuffer attachment teardown | `framebuffer()`, `framebuffer_texture_2d()`, `framebuffer_renderbuffer()` |

Draw calls (`draw_arrays`, `draw_elements_with_i32`, `draw_arrays_instanced`) and framebuffer creation/attachment (`create_framebuffer`, `framebuffer_texture2d`, `use_program`, `get_uniform_block_index`, `uniform_block_binding`, `bind_vertex_array`) are raw `web_sys::WebGl2RenderingContext` methods, called directly on the `GL` context returned by `context::from_canvas`/`retrieve_or_make` — minwebgl wraps resource creation/upload and attachment teardown (`clean`), not every context method. There is no top-level `draw` module.

## 🎯 Examples

- **[Trivial](../../../examples/minwebgl/trivial/)** - Minimal setup drawing a single point
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

Condensed from
[`examples/minwebgl/uniforms_ubo/src/main.rs`](../../../examples/minwebgl/uniforms_ubo/src/main.rs).
Block lookup/binding are raw `WebGl2RenderingContext` methods; `gl::ubo` supplies the upload and (behind the `diagnostics` feature) introspection helpers:

```rust,ignore
// Create the backing buffer, look up the block index, and bind it to a binding point
let color_buffer = gl::buffer::create( &gl )?;
let color_block_index = gl.get_uniform_block_index( &program, "ColorBlock" );
let color_block_point = 0;
gl.uniform_block_binding( &program, color_block_index, color_block_point );

// Optional: dump std140 layout info for debugging (feature = "diagnostics")
gl::ubo::diagnostic_info( &gl, &program, color_block_index ).debug_info();

// Every frame: re-upload the block's data (std140-laid-out Vec<f32> here)
gl::ubo::upload( &gl, &color_buffer, color_block_point, &color[ .. ], GL::DYNAMIC_DRAW );
```

### Error Handling

There is no separate debug-enable/get_error API. minwebgl reports failures through its own
[`WebglError`](src/context.rs) enum (`FailedToAllocateResource`, `CantUploadUniform`, `NotSupportedForType`, `DataType`, `DomError`, `ShaderError`, `MissingDataError`, `IdOutOfRange`, `Other`) — every fallible minwebgl call returns `Result< _, WebglError >`, propagated with `?` exactly as in the Basic Triangle Example above:

```rust,ignore
fn app_run() -> Result< (), gl::WebglError >
{
  let canvas = gl::canvas::make()?;
  let context = gl::context::from_canvas( &canvas )?;
  let program = gl::Program::new( context.clone(), vertex_src, fragment_src )?;
  // ...
  Ok( () )
}
```

