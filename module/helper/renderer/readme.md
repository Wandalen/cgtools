# 🎨 renderer

> **High-performance WebGL scene rendering engine with physically-based rendering**

A comprehensive 3D rendering system built specifically for WebAssembly and WebGL applications. Features modern PBR (Physically Based Rendering), post-processing effects, and efficient scene management for creating stunning real-time graphics in web browsers.

## ✨ Features

### 🎮 **Rendering Pipeline**
- **Physically Based Rendering (PBR)** - Industry-standard material system
- **Multi-Sample Anti-Aliasing (MSAA)** - Hardware-accelerated edge smoothing
- **HDR Rendering** - High dynamic range color pipeline
- **Post-Processing Stack** - Tone mapping, gamma correction, and effects

### 🏗️ **Scene Management**
- **glTF 2.0 Support** - Industry-standard 3D asset loading
- **Hierarchical Scenes** - Node-based scene graph with transformations
- **KHR Extensions** - Support for material extensions and advanced features
- **Asset Streaming** - Efficient loading of 3D models and textures

### 📷 **Camera System**
- **Perspective & Orthographic** - Multiple projection modes
- **Orbit Controls** - Interactive camera manipulation
- **View-Projection Matrices** - Optimized matrix calculations

### 🖼️ **Material System**
- **Metallic-Roughness Workflow** - Standard PBR material model
- **Normal Mapping** - Detailed surface rendering without additional geometry
- **Specular Extensions** - Advanced material properties via KHR_materials_specular
- **Texture Streaming** - Efficient texture memory management
- **Configurable Rendering Properties** - Per-material control of face culling, depth testing, and winding order


## 📦 Installation

Add to your `Cargo.toml`:
```toml
renderer = { workspace = true, features = ["webgl"] }
```

## 🧪 Canonical `gpu_hal` path ( `webgpu` feature )

Beside the WebGL renderer above, the `webgpu` feature builds the canonical
backend-portable opaque path ( `renderer::webgpu` ): PBR opaque pass + ACES
tone mapping, written once against the L1 `gpu_hal` crate and targeting any
of its backends — `GpuContext::new_webgpu( &canvas )`,
`GpuContext::new_webgl( &canvas )`, or, off-browser with the `native`
feature, `GpuContext::new_native( width, height )` over the machine's
Vulkan driver ( a software ICD such as lavapipe suffices ). Canonical
shaders are WGSL with GLSL 300 es twins; projections must match
`context.device.depth_range()`. The opaque path is pixel-verified in the
terminal on the native backend:

```sh
cargo nextest run -p renderer --features native
```

Scope today is the direct-lit opaque slice — IBL, shadows, skinning and the
loaders stay with the `webgl` renderer until strangled onto the HAL.

## 🚀 Quick Start

### Basic Rendering Setup

```rust,no_run
use minwebgl as gl;
use renderer::webgl::{ loaders, Camera, Renderer };

async fn setup() -> Result< (), gl::WebglError >
{
  // Setup the WebGL context — the renderer does MSAA internally,
  // so disable the built-in antialiasing
  let options = gl::context::ContextOptions::default().antialias( false );
  let canvas = gl::canvas::make()?;
  let gl = gl::context::from_canvas_with( &canvas, options )?;

  // HDR rendering needs float color buffers
  let _ = gl.get_extension( "EXT_color_buffer_float" )
  .expect( "EXT_color_buffer_float is not supported" );

  // Load a glTF scene
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();
  let gltf = loaders::gltf::load( &document, "static/model.glb", &gl ).await?;
  let scenes = gltf.scenes;
  scenes[ 0 ].borrow_mut().world_matrix_update();

  // Camera: eye, up, look-at center, aspect, vertical fov, near, far
  let eye = gl::math::F32x3::from( [ 0.0, 1.0, 3.0 ] );
  let up = gl::math::F32x3::from( [ 0.0, 1.0, 0.0 ] );
  let center = gl::math::F32x3::from( [ 0.0, 0.0, 0.0 ] );
  let aspect = canvas.width() as f32 / canvas.height() as f32;
  let mut camera = Camera::new( eye, up, center, aspect, 70.0f32.to_radians(), 0.1, 1000.0 );
  camera.window_size_set( [ canvas.width() as f32, canvas.height() as f32 ].into() );

  // Renderer with 4x MSAA, then a first frame into its internal HDR buffer
  let mut renderer = Renderer::new( &gl, canvas.width(), canvas.height(), 4 )?;
  renderer.render( &gl, &mut scenes[ 0 ].borrow_mut(), &camera )?;

  Ok( () )
}
# fn main() { let _ = setup; }
```

### Complete Render Loop with Post-Processing

```rust,no_run
use minwebgl as gl;
use renderer::webgl::{ Camera, Renderer, Scene };
use renderer::webgl::post_processing::{ Pass, SwapFramebuffer, ToneMappingAces, ToneMappingPass, ToSrgbPass };

// Create the pipeline pieces once and reuse them every frame:
//   SwapFramebuffer::new( &gl, width, height )
//   ToneMappingPass::< ToneMappingAces >::new( &gl )?
//   ToSrgbPass::new( &gl, true )?   // true = render to the screen
fn render_frame
(
  gl : &gl::WebGl2RenderingContext,
  renderer : &mut Renderer,
  scene : &mut Scene,
  camera : &Camera,
  swap_buffer : &mut SwapFramebuffer,
  tonemapping : &ToneMappingPass< ToneMappingAces >,
  to_srgb : &ToSrgbPass
) -> Result< (), gl::WebglError >
{
  // Render the scene into the renderer's internal HDR buffer
  renderer.render( gl, scene, camera )?;

  // Feed that HDR result into the post-processing chain
  swap_buffer.reset();
  swap_buffer.bind( gl );
  swap_buffer.input_set( renderer.main_texture() );

  // 1. Tone mapping ( HDR -> LDR, ACES )
  let tonemapped = tonemapping.render( gl, swap_buffer.input_get(), swap_buffer.output_get() )?;
  swap_buffer.output_set( tonemapped );
  swap_buffer.swap();

  // 2. Gamma correction ( final output to the screen )
  let _ = to_srgb.render( gl, swap_buffer.input_get(), swap_buffer.output_get() )?;

  Ok( () )
}
# fn main() { let _ = render_frame; }
```

See `examples/minwebgl/postprocessing` for the full interactive version of this pipeline.

## 📖 API Reference

### Core Components

| Component | Purpose | Key Methods |
|-----------|---------|-------------|
| `Renderer` | Main rendering engine | `new()`, `render()`, `main_texture()` |
| `SwapFramebuffer` | Post-processing helper | `bind()`, `input_set()`, `swap()` |
| `Scene` | 3D scene container | `world_matrix_update()` |
| `Camera` | Viewport and projection | Position, rotation, projection matrices |

### Post-Processing Effects

| Pass | Description | Use Case |
|------|-------------|----------|
| `ToneMappingPass<ToneMappingAces>` | ACES tone mapping | HDR to LDR conversion |
| `ToSrgbPass` | Gamma correction | Final color space conversion |
| Custom passes | User-defined effects | Bloom, blur, color grading |

### Asset Loading

```rust,no_run
use minwebgl as gl;
use renderer::webgl::loaders;

async fn load_assets
(
  document : &gl::web_sys::Document,
  gl : &gl::WebGl2RenderingContext
) -> Result< (), gl::WebglError >
{
  // Load glTF 2.0 files
  let gltf = loaders::gltf::load( document, "static/model.glb", gl ).await?;

  // Access scenes, meshes, materials
  let scene = &gltf.scenes[ 0 ];
  let materials = &gltf.materials;
  # let _ = ( scene, materials );
  Ok( () )
}
# fn main() { let _ = load_assets; }
```

### Features

Enable specific functionality:
```toml
renderer = { workspace = true, features = ["webgl", "full"] }
```

- `webgl` - WebGL rendering backend
- `webgpu` - Canonical `gpu_hal` opaque path ( browser targets )
- `native` - The same canonical path on the native wgpu backend ( terminal pixel tests, no browser )
- `full` - All features enabled

## 🎯 Use Cases

- **Game Development** - Real-time 3D games and interactive applications
- **Product Visualization** - High-quality product renders and configurators
- **Architectural Visualization** - Building and interior walkthroughs
- **Scientific Visualization** - Data visualization and simulation rendering
- **Art & Animation** - Creative tools and interactive art installations

## 🔧 Advanced Features

### Custom Materials
The renderer supports the KHR_materials_specular extension for advanced material properties beyond the standard metallic-roughness workflow.

When implementing the `Material` trait for custom materials:
- **`bind()`** must call `gl.active_texture(gl::TEXTURE0 + unit)` before each texture bind — this is the only method that should touch texture state.
- **`configure()`** sets up texture sampler uniform locations once at program creation time.
- **`upload_on_state_change()`** uploads uniform values; use `needs_update()` / `set_needs_update(false)` with `Cell<bool>` to avoid redundant uploads.
- IBL textures occupy units starting from `ibl_base_texture_unit()` (3 consecutive units). Custom materials should avoid those units.

### Performance Optimization
- **Shader program caching** - Materials with identical shader source share a single compiled GPU program
- **Draw call grouping** - Primitives are sorted by shader program to minimize state switches
- **Dirty-flag material updates** - Uniform uploads are skipped when material state hasn't changed
- Multi-sample anti-aliasing (MSAA) for edge smoothing
- HDR rendering pipeline for realistic lighting
- Efficient memory management for large scenes
- WebAssembly-optimized rendering paths

## 📐 Design Documentation

Typed design docs live in [`docs/`](docs/definition/readme.md): the crate's
invariants (GPU-resolved visibility with OIT, PBR metallic-roughness
baseline, HDR-internal pipeline), confirmed pitfalls
(`EXT_color_buffer_float` is required but never enabled by the crate), and
feature hubs for the PBR core, image-based lighting, and shadow mapping.

## 📚 References & Research

#### PBR
- [Real Shading in Unreal Engine 4]
- [Background: Physics and Math of Shading]
- [Moving Frostbite to Physically Based Rendering 2.0]
- [Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs]
- [Importance Sampling techniques for GGX with Smith Masking-Shadowing: Part 1]
- [Importance Sampling techniques for GGX with Smith Masking-Shadowing: Part 2]
- [Microfacet Models for Refraction through Rough Surfaces]
- [PBR Diffuse Lighting for GGX+Smith Microsurfaces]
- [Sampling Microfacet BRDF]
- [Notes on importance sampling]
- [Article - Physically Based Rendering - Cook–Torrance]
- [Vulkan-glTF-PBR]
-

#### Normal mapping
- [Normals and the Inverse Transpose, Part 1: Grassmann Algebra]
- [Normals and the Inverse Transpose, Part 2: Dual Spaces]
- [Normal Mapping Without Precomputed Tangents]

#### KHR Extensions
- [KHR_materials_specular]

[Real Shading in Unreal Engine 4]: https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf
[Background: Physics and Math of Shading]: https://blog.selfshadow.com/publications/s2013-shading-course/hoffman/s2013_pbs_physics_math_notes.pdf
[Moving Frostbite to Physically Based Rendering 2.0]: https://web.archive.org/web/20160702002225/http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr_v2.pdf
[Understanding the Masking-Shadowing Function in Microfacet-Based BRDFs]: https://inria.hal.science/hal-00942452v1/document
[Importance Sampling techniques for GGX with Smith Masking-Shadowing: Part 1]: https://schuttejoe.github.io/post/ggximportancesamplingpart1/
[Importance Sampling techniques for GGX with Smith Masking-Shadowing: Part 2]: https://schuttejoe.github.io/post/ggximportancesamplingpart2/
[Microfacet Models for Refraction through Rough Surfaces]: https://www.cs.cornell.edu/~srm/publications/EGSR07-btdf.pdf
[PBR Diffuse Lighting for GGX+Smith Microsurfaces]: https://ubm-twvideo01.s3.amazonaws.com/o1/vault/gdc2017/Presentations/Hammon_Earl_PBR_Diffuse_Lighting.pdf
[Sampling Microfacet BRDF]: https://agraphicsguynotes.com/posts/sample_microfacet_brdf/
[Notes on importance sampling]: https://www.tobias-franke.eu/log/2014/03/30/notes_on_importance_sampling.html
[How Is The NDF Really Defined?]: https://www.reedbeta.com/blog/hows-the-ndf-really-defined/
[Article - Physically Based Rendering - Cook–Torrance]: http://www.codinglabs.net/article_physically_based_rendering_cook_torrance.aspx

[Normals and the Inverse Transpose, Part 1: Grassmann Algebra]: https://www.reedbeta.com/blog/normals-inverse-transpose-part-1/
[Normals and the Inverse Transpose, Part 2: Dual Spaces]: https://www.reedbeta.com/blog/normals-inverse-transpose-part-2/
[Normal Mapping Without Precomputed Tangents]: http://www.thetenthplanet.de/archives/1180

[KHR_materials_specular]:  https://github.com/KhronosGroup/glTF/blob/main/extensions/2.0/Khronos/KHR_materials_specular/README.md
[Vulkan-glTF-PBR]: https://github.com/SaschaWillems/Vulkan-glTF-PBR/blob/master/data/shaders/genbrdflut.frag
[Image Based Lighting with Multiple Scattering]: https://bruop.github.io/ibl/
