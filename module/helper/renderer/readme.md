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
- **Dynamic Text Engraving** - Node-scoped, runtime-editable engraved text (relief normal, groove roughness/darkening) on `PbrMaterial`; see `src/webgl/engraving/`


## 📦 Installation

Add to your `Cargo.toml`:
```toml
renderer = { workspace = true, features = ["webgl"] }
```

## 🚀 Quick Start

### Basic Rendering Setup

```rust
use minwebgl as gl;
use renderer::webgl::{loaders, Renderer, SwapFramebuffer};
use renderer::webgl::post_processing::{ToneMappingPass, ToSrgbPass, ToneMappingAces};

async fn setup_renderer() -> Result<(), Box<dyn std::error::Error>> {
  // Setup WebGL context
  let window = gl::web_sys::window().unwrap();
  let document = window.document().unwrap();
  let canvas = gl::canvas::make()?;

  // Disable antialiasing (renderer uses MSAA internally)
  let options = gl::context::ContexOptions::default().antialias(false);
  let gl = gl::context::from_canvas_with(&canvas, options)?;

  // Enable HDR rendering
  gl.get_extension("EXT_color_buffer_float")
    .expect("HDR textures not supported");

  // Create renderer with 4x MSAA
  let renderer = Renderer::new(&gl, canvas.width(), canvas.height(), 4);

  // Load 3D scene
  let gltf = loaders::gltf::load(&document, "assets/model.gltf", &gl).await?;
  let scene = &gltf.scenes[0];

  Ok(())
}
```

### Complete Render Loop with Post-Processing

```rust
async fn render_frame(
  renderer: &Renderer,
  scene: &mut Scene,
  camera: &Camera,
  gl: &WebGl2RenderingContext,
) -> Result<(), Box<dyn std::error::Error>> {
  // Setup post-processing pipeline
  let mut swap_buffer = SwapFramebuffer::new(gl, canvas.width(), canvas.height());
  let tonemapping = ToneMappingPass::<ToneMappingAces>::new(
    gl, canvas.width(), canvas.height()
  )?;
  let to_srgb = ToSrgbPass::new(gl, true)?; // Render to screen

  // Update scene transformations
  scene.update_world_matrix();

  // Render scene to HDR buffer
  renderer.render(gl, scene, camera)?;

  // Post-processing pipeline
  swap_buffer.reset();
  swap_buffer.bind(gl);
  swap_buffer.set_input(renderer.main_texture());

  // 1. Tone mapping (HDR -> LDR)
  let tonemapped = tonemapping.render(
    gl,
    swap_buffer.get_input(),
    swap_buffer.get_output()
  )?;

  swap_buffer.set_output(tonemapped);
  swap_buffer.swap();

  // 2. Gamma correction (final output to screen)
  to_srgb.render(gl, swap_buffer.get_input(), swap_buffer.get_output())?;

  Ok(())
}
```

## 📖 API Reference

### Core Components

| Component | Purpose | Key Methods |
|-----------|---------|-------------|
| `Renderer` | Main rendering engine | `new()`, `render()`, `main_texture()` |
| `SwapFramebuffer` | Post-processing helper | `bind()`, `set_input()`, `swap()` |
| `Scene` | 3D scene container | `update_world_matrix()` |
| `Camera` | Viewport and projection | Position, rotation, projection matrices |

### Post-Processing Effects

| Pass | Description | Use Case |
|------|-------------|----------|
| `ToneMappingPass<ToneMappingAces>` | ACES tone mapping | HDR to LDR conversion |
| `ToSrgbPass` | Gamma correction | Final color space conversion |
| Custom passes | User-defined effects | Bloom, blur, color grading |

### Asset Loading

```rust
use renderer::webgl::loaders;

// Load glTF 2.0 files
let gltf = loaders::gltf::load(&document, "model.gltf", &gl).await?;

// Access scenes, meshes, materials
let scene = &gltf.scenes[0];
let materials = &gltf.materials;
```

### Features

Enable specific functionality:
```toml
renderer = { workspace = true, features = ["webgl", "full"] }
```

- `webgl` - WebGL rendering backend
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
- **`as_any()` / `as_any_mut()`** let calling code downcast a `Box<dyn Material>` back to its concrete type — used by the engraving system to reach `PbrMaterial`-specific setters after resolving a node by name.

### Dynamic Text Engraving
`src/webgl/engraving/` maps glTF node names to an engraving zone (a dedicated UV channel, aspect ratio, character limit and font whitelist — see `engraving_config.schema.json`), and wires user-supplied text all the way to the GPU:

1. `EngravingConfig::from_json` parses `engraving_config.json`.
2. `EngravingSession::build` resolves each configured node against a loaded `Scene`, allocates one offscreen canvas + one GPU mask texture per node, and turns on `PbrMaterial`'s `USE_ENGRAVING` shader path.
3. `EngravingSession::set_text` (async — it awaits `document.fonts.load()`) rasterizes white-on-black text once per mip level (re-drawn at each level's own resolution, not box-filtered down from level 0 — a binary text mask aliases badly under a naive `generateMipmap`, and that aliasing feeds straight into the shader's derivative-based relief normal) and uploads the whole chain via `texSubImage2D` per level — no reallocation or shader recompilation, so it's cheap to call on every keystroke.

Font sizing per node follows one of three `SizingMode`s (`EngravingNodeConfig::resolved_sizing_mode`): `PHYSICAL` and `HYBRID` derive a target size in canvas px from `defaultFontSizeMm / stripHeightMm` (HYBRID shrinks toward `minFontSizeMm` on overflow instead of rejecting it outright), `RELATIVE` either uses a fixed `fontSizeRatio` of canvas height or auto-fits the largest size that fits the padded bounds. A node with none of the physical fields set resolves to `RELATIVE` automatically, so `sizingMode` is opt-in, not required.

`main.frag`'s `USE_ENGRAVING` block bounds-checks the mask UV against `[0, 1]`, perturbs the normal from a UV-space surface gradient (explicit-LOD `textureLod` central differences along the mask's U/V axes, projected onto the tangent frame — a first-order bump-mapping approximation of a beveled groove), and pushes roughness/albedo towards a matte, slightly darkened look inside the mask — without touching the base metal's hue. Sampling the gradient at an explicit LOD (rather than differentiating a filtered `texture()` fetch via `dFdx`/`dFdy`) keeps it well-behaved at any distance or viewing angle, so no separate fade-out stage is needed.

See `src/webgl/engraving/artist_guide.md` for the 3D-artist-facing asset prep guide (mesh/UV requirements, export settings, `engraving_config.json` fields) covering the Blender -> glTF side of this pipeline.

### Performance Optimization
- **Shader program caching** - Materials with identical shader source share a single compiled GPU program
- **Draw call grouping** - Primitives are sorted by shader program to minimize state switches
- **Dirty-flag material updates** - Uniform uploads are skipped when material state hasn't changed
- Multi-sample anti-aliasing (MSAA) for edge smoothing
- HDR rendering pipeline for realistic lighting
- Efficient memory management for large scenes
- WebAssembly-optimized rendering paths

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
