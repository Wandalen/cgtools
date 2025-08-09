# 🎭 glTF PBR Viewer

> **Production-quality 3D model viewer with physically-based rendering**

A comprehensive glTF 2.0 viewer implementing industry-standard physically-based rendering (PBR) with image-based lighting (IBL), HDR tone mapping, and material extensions. Load and explore 3D models with photorealistic material rendering in your browser.

![glTF Viewer Screenshot](./showcase.jpg)

## ✨ Features

### 📦 **glTF 2.0 Support**
- **Complete Parsing** - Buffers, images, cameras, scenes, materials, meshes
- **Material System** - Full PBR material support with texture maps
- **Geometry Handling** - Mesh primitives, vertex attributes, tangent generation
- **Scene Management** - Multi-scene support with proper hierarchy

### 🎨 **Physically-Based Rendering**
- **Metallic-Roughness Workflow** - Industry-standard material pipeline
- **Image-Based Lighting** - Realistic environment lighting with HDR
- **Normal Mapping** - Surface detail enhancement
- **Occlusion Mapping** - Ambient shadowing effects
- **HDR Tone Mapping** - High dynamic range color processing

### 🔧 **Advanced Graphics**
- **WebGL 2.0 Optimized** - Hardware-accelerated rendering
- **Tangent Space Generation** - Automatic tangent vector computation
- **Multi-Pass Rendering** - Complex lighting calculations
- **Texture Management** - Efficient GPU memory usage

## 🚀 Quick Start

### Prerequisites
- WebGL 2.0 compatible browser
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Run the Example
```bash
# Navigate to glTF viewer example
cd examples/minwebgl/gltf_viewer

# Install trunk if needed
cargo install trunk

# Serve the example
trunk serve --release
```

Open http://localhost:8080 and load your glTF models to explore PBR rendering.

## 📋 Implementation Status

### ✅ **Fully Implemented**

#### glTF Parsing
- ✅ **Buffers** - Binary data loading and management
- ✅ **Images** - Texture loading with format support
- ✅ **Cameras** - Perspective and orthographic projection
- ✅ **Scenes** - Node hierarchy and transformations
- ✅ **Materials** - PBR material properties
- ✅ **Meshes** - Geometry primitives and attributes
- ✅ **Tangents** - Normal mapping support

#### PBR Shading
- ✅ **Base Color Texture** - Diffuse albedo maps
- ✅ **Metallic Texture** - Metal/non-metal material classification
- ✅ **Roughness Texture** - Surface roughness control
- ✅ **Normal Texture** - Surface detail enhancement
- ✅ **Occlusion Texture** - Ambient shadowing
- ✅ **Image-Based Lighting** - Environment map lighting
- ✅ **HDR Tone Mapping** - High dynamic range processing

#### Rendering Features
- ✅ **Multiple Scenes** - Switch between different scene configurations
- ✅ **KHR_materials_specular** - Extended material properties

### ⏳ **Planned Features**

#### glTF Extensions
- ❌ **Sparse Accessors** - Memory-efficient geometry data
- ❌ **Animations** - Keyframe-based object animation
- ❌ **Skins and Bones** - Skeletal animation system
- ❌ **Morph Targets** - Mesh deformation/blendshapes

#### Advanced Rendering
- ❌ **Emission Texture** - Self-illuminated materials
- ❌ **Bone Transformations** - Skeletal animation rendering
- ❌ **Multisampling** - Hardware anti-aliasing
- ❌ **Multiple Cameras** - Camera switching interface

#### User Interface
- ❌ **Scene Switching** - UI for multiple scene selection
- ❌ **Camera Controls** - Interactive camera manipulation
- ❌ **Tone Mapping Controls** - HDR adjustment interface
- ❌ **GPU Statistics** - Performance monitoring display

## 🔧 Technical Implementation

### PBR Shader Architecture

The viewer implements the metallic-roughness PBR workflow:

```glsl
// Fragment shader PBR implementation
vec3 calculatePBR(vec3 albedo, float metallic, float roughness, vec3 normal, vec3 viewDir, vec3 lightDir) {
  // Fresnel reflectance at normal incidence
  vec3 F0 = mix(vec3(0.04), albedo, metallic);
  
  // Cook-Torrance BRDF components
  vec3 F = fresnelSchlick(max(dot(normal, viewDir), 0.0), F0);
  float NDF = distributionGGX(normal, viewDir, roughness);
  float G = geometrySmith(normal, viewDir, lightDir, roughness);
  
  // BRDF calculation
  vec3 numerator = NDF * G * F;
  float denominator = max(4.0 * dot(normal, viewDir) * dot(normal, lightDir), 0.001);
  vec3 specular = numerator / denominator;
  
  // Diffuse component
  vec3 kS = F;
  vec3 kD = vec3(1.0) - kS;
  kD *= 1.0 - metallic;
  
  return (kD * albedo / PI + specular) * radiance * dot(normal, lightDir);
}
```

### Image-Based Lighting

Environmental lighting using pre-filtered HDR environment maps:

```glsl
// IBL ambient lighting calculation
vec3 calculateIBL(vec3 albedo, float metallic, float roughness, vec3 normal, vec3 viewDir) {
  vec3 F0 = mix(vec3(0.04), albedo, metallic);
  vec3 F = fresnelSchlickRoughness(max(dot(normal, viewDir), 0.0), F0, roughness);
  
  vec3 kS = F;
  vec3 kD = 1.0 - kS;
  kD *= 1.0 - metallic;
  
  // Diffuse IBL
  vec3 irradiance = texture(irradianceMap, normal).rgb;
  vec3 diffuse = irradiance * albedo;
  
  // Specular IBL
  vec3 reflectionDir = reflect(-viewDir, normal);
  const float MAX_REFLECTION_LOD = 4.0;
  vec3 prefilteredColor = textureLod(prefilterMap, reflectionDir, roughness * MAX_REFLECTION_LOD).rgb;
  
  vec2 brdf = texture(brdfLUT, vec2(max(dot(normal, viewDir), 0.0), roughness)).rg;
  vec3 specular = prefilteredColor * (F * brdf.x + brdf.y);
  
  return kD * diffuse + specular;
}
```

### HDR Tone Mapping

Converting high dynamic range to displayable range:

```glsl
// ACES tone mapping operator
vec3 ACESFilm(vec3 x) {
  float a = 2.51;
  float b = 0.03;
  float c = 2.43;
  float d = 0.59;
  float e = 0.14;
  return clamp((x * (a * x + b)) / (x * (c * x + d) + e), 0.0, 1.0);
}

// Main tone mapping function
vec3 toneMap(vec3 hdrColor, float exposure) {
  // Exposure adjustment
  vec3 exposed = hdrColor * exposure;
  
  // Apply tone mapping
  vec3 toneMapped = ACESFilm(exposed);
  
  // Gamma correction
  return pow(toneMapped, vec3(1.0/2.2));
}
```

## 🎯 Technical Challenges & Solutions

### WebGL Limitations

#### Mipmap Generation
**Issue**: WebGL requires complete mipmap chains for sampling
**Solution**: Generate missing mip levels programmatically

```rust
// Ensure complete mipmap chain
fn ensure_complete_mipmaps(gl: &WebGl2RenderingContext, texture: &WebGlTexture) {
  gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(texture));
  gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);
}
```

#### HDR Image Loading
**Issue**: zune-hdr crate has bugs with small HDR images
**Solution**: Use development version with fixes

```toml
[dependencies]
zune-hdr = { git = "https://github.com/etemesi254/zune-image", branch = "main" }
```

#### BRDF Edge Artifacts
**Issue**: Division by small numbers causes flickering at model edges
**Solution**: Clamp denominator to avoid numerical instability

```glsl
// Avoid division by zero
float denominator = max(4.0 * dotVN * dotNL, 0.001);
```

## 📚 PBR Theory and Implementation

### Material Properties

#### Metallic-Roughness Workflow
- **Base Color** - Surface albedo (diffuse color for non-metals)
- **Metallic** - Binary metal/dielectric classification (0 or 1)
- **Roughness** - Surface microsurface roughness (0 = mirror, 1 = completely rough)

#### Physically-Based Principles
```glsl
// Energy conservation: diffuse + specular ≤ 1
vec3 kS = fresnel; // Specular contribution
vec3 kD = (1.0 - kS) * (1.0 - metallic); // Diffuse contribution

// Metals have no diffuse reflection
vec3 F0 = mix(vec3(0.04), baseColor, metallic);
```

### Advanced Lighting Models

#### Fresnel Reflection
```glsl
vec3 fresnelSchlick(float cosTheta, vec3 F0) {
  return F0 + (1.0 - F0) * pow(clamp(1.0 - cosTheta, 0.0, 1.0), 5.0);
}
```

#### Normal Distribution Function (GGX)
```glsl
float distributionGGX(vec3 N, vec3 H, float roughness) {
  float a = roughness * roughness;
  float a2 = a * a;
  float NdotH = max(dot(N, H), 0.0);
  float NdotH2 = NdotH * NdotH;
  
  float num = a2;
  float denom = (NdotH2 * (a2 - 1.0) + 1.0);
  denom = PI * denom * denom;
  
  return num / denom;
}
```

#### Geometry Masking-Shadowing
```glsl
float geometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
  float NdotV = max(dot(N, V), 0.0);
  float NdotL = max(dot(N, L), 0.0);
  float ggx2 = geometrySchlickGGX(NdotV, roughness);
  float ggx1 = geometrySchlickGGX(NdotL, roughness);
  
  return ggx1 * ggx2;
}
```

## 🎮 Usage Examples

### Loading Custom Models

```rust
// Load glTF from URL
async fn load_model(url: &str) -> Result<GltfScene, JsValue> {
  let response = fetch(url).await?;
  let bytes = response.array_buffer().await?;
  
  let gltf_data = parse_gltf(&bytes)?;
  let scene = build_render_scene(&gltf_data)?;
  
  Ok(scene)
}
```

### Material Customization

```rust
// Override material properties
struct MaterialOverride {
  base_color: [f32; 4],
  metallic_factor: f32,
  roughness_factor: f32,
  normal_scale: f32,
}

fn apply_material_override(material: &mut PbrMaterial, override_props: &MaterialOverride) {
  material.base_color_factor = override_props.base_color;
  material.metallic_factor = override_props.metallic_factor;
  material.roughness_factor = override_props.roughness_factor;
  material.normal_texture_scale = override_props.normal_scale;
}
```

## 🔗 Educational Resources

### PBR Theory
- **[Real Shading in Unreal Engine 4](https://blog.selfshadow.com/publications/s2013-shading-course/karis/s2013_pbs_epic_notes_v2.pdf)** - Industry standard PBR implementation
- **[Background: Physics and Math of Shading](https://blog.selfshadow.com/publications/s2013-shading-course/hoffman/s2013_pbs_physics_math_notes.pdf)** - Mathematical foundations
- **[Moving Frostbite to PBR 2.0](https://web.archive.org/web/20160702002225/http://www.frostbite.com/wp-content/uploads/2014/11/course_notes_moving_frostbite_to_pbr_v2.pdf)** - Production pipeline insights

### Advanced Techniques
- **[Understanding Masking-Shadowing](https://inria.hal.science/hal-00942452v1/document)** - Geometry function theory
- **[GGX Importance Sampling Part 1](https://schuttejoe.github.io/post/ggximportancesamplingpart1/)** - Monte Carlo integration
- **[GGX Importance Sampling Part 2](https://schuttejoe.github.io/post/ggximportancesamplingpart2/)** - Advanced sampling techniques

### Implementation Guides
- **[Sampling Microfacet BRDF](https://agraphicsguynotes.com/posts/sample_microfacet_brdf/)** - Practical BRDF implementation
- **[Normal Mapping Without Tangents](http://www.thetenthplanet.de/archives/1180)** - Alternative normal mapping approach
- **[Vulkan glTF PBR Reference](https://github.com/SaschaWillems/Vulkan-glTF-PBR)** - Complete reference implementation

## 🛠️ Development Notes

### Performance Optimization
- **Texture Atlas Packing** - Combine multiple textures for fewer draw calls
- **Frustum Culling** - Skip off-screen objects
- **Level of Detail** - Distance-based mesh simplification
- **Batch Rendering** - Group similar materials together

### Memory Management
```rust
// Efficient texture loading
struct TextureManager {
  cache: HashMap<String, WebGlTexture>,
  gl: Rc<WebGl2RenderingContext>,
}

impl TextureManager {
  fn load_or_get(&mut self, path: &str) -> Result<WebGlTexture, JsValue> {
    if let Some(texture) = self.cache.get(path) {
      return Ok(texture.clone());
    }
    
    let texture = self.load_texture_from_path(path)?;
    self.cache.insert(path.to_string(), texture.clone());
    Ok(texture)
  }
}
```

### Debugging Tools
```rust
// Debug material properties
fn debug_render_material_channels(&self, material: &PbrMaterial) {
  // Visualize individual texture channels
  self.render_texture_preview(&material.base_color_texture, "Base Color");
  self.render_texture_preview(&material.normal_texture, "Normal Map");
  self.render_texture_preview(&material.metallic_roughness_texture, "Metallic/Roughness");
  self.render_texture_preview(&material.occlusion_texture, "Occlusion");
}
```

## 🤝 Contributing

Part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT