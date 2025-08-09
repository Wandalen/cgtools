# 🏦 Universal OBJ Model Viewer

> **Production-ready 3D model viewer with PBR shading and material support**

A comprehensive OBJ model viewer implementing industry-standard physically-based rendering (PBR) with full material support. This viewer is designed to handle a wide variety of 3D models with automatic optimization, diagnostic tools, and professional-quality shading.

![OBJ Viewer Example](./showcase.jpg)

## ✨ Features

### 📎 **OBJ Format Support**
- **Complete OBJ Parsing** - Vertices, faces, normals, texture coordinates
- **MTL Material Loading** - Full material library support
- **Multi-Object Models** - Handle complex scenes with multiple objects
- **Large Model Optimization** - Efficient loading of high-polygon models

### 🎨 **Material System**
- **PBR Materials** - Metallic-roughness workflow implementation
- **Texture Support** - Diffuse, normal, metallic, roughness, occlusion maps
- **Material Properties** - Ambient, diffuse, specular, shininess support
- **Fallback Shading** - Graceful degradation for missing materials

### 🔧 **Technical Features**
- **WebGL 2.0 Optimized** - Hardware-accelerated rendering
- **Diagnostic Tools** - Built-in model analysis and debugging
- **Performance Monitoring** - Load time and render performance tracking
- **Memory Management** - Efficient GPU resource usage

## 🚀 Quick Start

### Prerequisites
- WebGL 2.0 compatible browser
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Loading Your Model

1. **Prepare Model Files**
   ```
   assets/
   └── your-model/
       ├── model.obj          # Main geometry file
       ├── model.mtl          # Material library
       └── textures/
           ├── diffuse.jpg    # Base color textures
           ├── normal.jpg     # Normal maps
           └── roughness.jpg  # PBR textures
   ```

2. **Update Configuration**
   ```rust
   // In main.rs, specify your model paths
   let mtl_path = "your-model";                    // Material folder
   let texture_path = "your-model/textures";       // Texture folder  
   let obj_path = "your-model/model.obj";          // OBJ file path
   ```

3. **Build and Run**
   ```bash
   # Navigate to OBJ viewer
   cd examples/minwebgl/obj_viewer
   
   # Important: Use release mode for large models
   trunk serve --release
   ```

Open http://localhost:8080 to view your 3D model.

## 🔧 Technical Implementation

### OBJ Parser Architecture

Robust OBJ file parsing with error handling and optimization:

```rust
// OBJ model structure
struct ObjModel {
  vertices: Vec<Vec3>,
  normals: Vec<Vec3>,
  tex_coords: Vec<Vec2>,
  faces: Vec<Face>,
  materials: HashMap<String, Material>,
  objects: Vec<Object>,
}

// Face definition with material reference
struct Face {
  vertex_indices: [usize; 3],
  normal_indices: Option<[usize; 3]>,
  tex_coord_indices: Option<[usize; 3]>,
  material_name: Option<String>,
}

// Efficient OBJ parsing
impl ObjModel {
  fn parse_from_string(obj_data: &str, mtl_data: Option<&str>) -> Result<Self, ParseError> {
    let mut model = ObjModel::new();
    
    for line in obj_data.lines() {
      let tokens: Vec<&str> = line.split_whitespace().collect();
      if tokens.is_empty() { continue; }
      
      match tokens[0] {
        "v" => model.parse_vertex(&tokens)?,
        "vn" => model.parse_normal(&tokens)?,
        "vt" => model.parse_tex_coord(&tokens)?,
        "f" => model.parse_face(&tokens)?,
        "usemtl" => model.set_current_material(&tokens[1]),
        "o" => model.start_new_object(&tokens[1]),
        _ => {} // Skip unknown commands
      }
    }
    
    // Parse materials if available
    if let Some(mtl) = mtl_data {
      model.parse_materials(mtl)?;
    }
    
    model.finalize_parsing()
  }
}
```

### Material System Implementation

```rust
// Comprehensive material definition
#[derive(Debug, Clone)]
struct Material {
  // PBR properties
  base_color: Vec3,
  metallic: f32,
  roughness: f32,
  
  // Traditional properties (fallback)
  ambient: Vec3,
  diffuse: Vec3,
  specular: Vec3,
  shininess: f32,
  
  // Texture maps
  diffuse_map: Option<String>,
  normal_map: Option<String>,
  metallic_map: Option<String>,
  roughness_map: Option<String>,
  occlusion_map: Option<String>,
  
  // Rendering properties
  opacity: f32,
  double_sided: bool,
}

// MTL file parsing
impl Material {
  fn parse_mtl_file(mtl_data: &str) -> Result<HashMap<String, Material>, ParseError> {
    let mut materials = HashMap::new();
    let mut current_material: Option<Material> = None;
    let mut current_name: Option<String> = None;
    
    for line in mtl_data.lines() {
      let tokens: Vec<&str> = line.split_whitespace().collect();
      if tokens.is_empty() { continue; }
      
      match tokens[0] {
        "newmtl" => {
          // Save previous material
          if let (Some(name), Some(material)) = (current_name.take(), current_material.take()) {
            materials.insert(name, material);
          }
          
          // Start new material
          current_name = Some(tokens[1].to_string());
          current_material = Some(Material::default());
        },
        "Ka" => { // Ambient color
          if let Some(ref mut mat) = current_material {
            mat.ambient = parse_color(&tokens[1..]);
          }
        },
        "Kd" => { // Diffuse color
          if let Some(ref mut mat) = current_material {
            mat.diffuse = parse_color(&tokens[1..]);
            mat.base_color = mat.diffuse; // Use as PBR base color
          }
        },
        "Ks" => { // Specular color
          if let Some(ref mut mat) = current_material {
            mat.specular = parse_color(&tokens[1..]);
          }
        },
        "Ns" => { // Shininess
          if let Some(ref mut mat) = current_material {
            mat.shininess = tokens[1].parse().unwrap_or(1.0);
            // Convert shininess to roughness approximation
            mat.roughness = 1.0 - (mat.shininess / 1000.0).min(1.0);
          }
        },
        "map_Kd" => { // Diffuse texture
          if let Some(ref mut mat) = current_material {
            mat.diffuse_map = Some(tokens[1].to_string());
          }
        },
        "map_Bump" | "bump" => { // Normal map
          if let Some(ref mut mat) = current_material {
            mat.normal_map = Some(tokens[1].to_string());
          }
        },
        _ => {} // Handle other material properties
      }
    }
    
    // Save final material
    if let (Some(name), Some(material)) = (current_name, current_material) {
      materials.insert(name, material);
    }
    
    Ok(materials)
  }
}
```

### PBR Shader Implementation

```glsl
// Vertex shader for PBR rendering
#version 300 es
precision mediump float;

in vec3 position;
in vec3 normal;
in vec2 texCoord;

uniform mat4 modelMatrix;
uniform mat4 viewMatrix;
uniform mat4 projectionMatrix;
uniform mat4 normalMatrix;

out vec3 worldPosition;
out vec3 worldNormal;
out vec2 vTexCoord;

void main() {
  vec4 worldPos = modelMatrix * vec4(position, 1.0);
  worldPosition = worldPos.xyz;
  worldNormal = (normalMatrix * vec4(normal, 0.0)).xyz;
  vTexCoord = texCoord;
  
  gl_Position = projectionMatrix * viewMatrix * worldPos;
}
```

```glsl
// Fragment shader with PBR implementation
#version 300 es
precision mediump float;

in vec3 worldPosition;
in vec3 worldNormal;
in vec2 vTexCoord;

// Material uniforms
uniform vec3 baseColor;
uniform float metallic;
uniform float roughness;
uniform float opacity;

// Texture samplers
uniform sampler2D diffuseMap;
uniform sampler2D normalMap;
uniform sampler2D metallicMap;
uniform sampler2D roughnessMap;
uniform sampler2D occlusionMap;

// Texture availability flags
uniform bool hasDiffuseMap;
uniform bool hasNormalMap;
uniform bool hasMetallicMap;
uniform bool hasRoughnessMap;
uniform bool hasOcclusionMap;

// Lighting uniforms
uniform vec3 lightDirection;
uniform vec3 lightColor;
uniform vec3 cameraPosition;

out vec4 fragColor;

// PBR lighting calculation
vec3 calculatePBR(vec3 albedo, float metallic, float roughness, vec3 normal, vec3 viewDir, vec3 lightDir) {
  // Fresnel reflectance at normal incidence
  vec3 F0 = mix(vec3(0.04), albedo, metallic);
  
  // Half vector
  vec3 halfVector = normalize(lightDir + viewDir);
  
  // Distribution (GGX)
  float alpha = roughness * roughness;
  float alpha2 = alpha * alpha;
  float NdotH = max(dot(normal, halfVector), 0.0);
  float NdotH2 = NdotH * NdotH;
  float denom = NdotH2 * (alpha2 - 1.0) + 1.0;
  float D = alpha2 / (3.14159265 * denom * denom);
  
  // Geometry function
  float NdotV = max(dot(normal, viewDir), 0.0);
  float NdotL = max(dot(normal, lightDir), 0.0);
  float k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
  float G1V = NdotV / (NdotV * (1.0 - k) + k);
  float G1L = NdotL / (NdotL * (1.0 - k) + k);
  float G = G1V * G1L;
  
  // Fresnel
  vec3 F = F0 + (1.0 - F0) * pow(clamp(1.0 - max(dot(halfVector, viewDir), 0.0), 0.0, 1.0), 5.0);
  
  // BRDF
  vec3 numerator = D * G * F;
  float denominator = max(4.0 * NdotV * NdotL, 0.001);
  vec3 specular = numerator / denominator;
  
  // Energy conservation
  vec3 kS = F;
  vec3 kD = vec3(1.0) - kS;
  kD *= 1.0 - metallic;
  
  vec3 irradiance = lightColor * NdotL;
  return (kD * albedo / 3.14159265 + specular) * irradiance;
}

void main() {
  // Sample textures
  vec3 albedo = hasDiffuseMap ? texture(diffuseMap, vTexCoord).rgb : baseColor;
  float metallicValue = hasMetallicMap ? texture(metallicMap, vTexCoord).r : metallic;
  float roughnessValue = hasRoughnessMap ? texture(roughnessMap, vTexCoord).r : roughness;
  
  // Normal mapping
  vec3 normal = normalize(worldNormal);
  if (hasNormalMap) {
    vec3 tangentNormal = texture(normalMap, vTexCoord).rgb * 2.0 - 1.0;
    // Apply normal map (simplified - full implementation needs tangent space)
    normal = normalize(normal + tangentNormal * 0.1);
  }
  
  // Ambient occlusion
  float ao = hasOcclusionMap ? texture(occlusionMap, vTexCoord).r : 1.0;
  
  // Lighting calculation
  vec3 viewDir = normalize(cameraPosition - worldPosition);
  vec3 lightDir = normalize(-lightDirection);
  
  vec3 color = calculatePBR(albedo, metallicValue, roughnessValue, normal, viewDir, lightDir);
  
  // Apply ambient occlusion
  color *= ao;
  
  // Add simple ambient lighting
  color += albedo * 0.1 * ao;
  
  fragColor = vec4(color, opacity);
}
```

## 📊 Performance Optimization

### Large Model Handling

```rust
// Efficient vertex buffer creation
struct OptimizedMesh {
  vertex_buffer: WebGlBuffer,
  index_buffer: WebGlBuffer,
  vertex_count: usize,
  index_count: usize,
  material_groups: Vec<MaterialGroup>,
}

struct MaterialGroup {
  material_name: String,
  start_index: usize,
  index_count: usize,
}

impl OptimizedMesh {
  fn from_obj_model(gl: &WebGl2RenderingContext, model: &ObjModel) -> Result<Self, JsValue> {
    // Convert OBJ data to optimized vertex format
    let vertices = Self::flatten_vertices(model);
    let indices = Self::generate_indices(model);
    let material_groups = Self::group_by_material(model);
    
    // Create GPU buffers
    let vertex_buffer = Self::create_vertex_buffer(gl, &vertices)?;
    let index_buffer = Self::create_index_buffer(gl, &indices)?;
    
    Ok(Self {
      vertex_buffer,
      index_buffer,
      vertex_count: vertices.len(),
      index_count: indices.len(),
      material_groups,
    })
  }
  
  fn render(&self, gl: &WebGl2RenderingContext, materials: &HashMap<String, Material>) {
    // Bind buffers
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vertex_buffer));
    gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.index_buffer));
    
    // Render by material groups to minimize state changes
    for group in &self.material_groups {
      if let Some(material) = materials.get(&group.material_name) {
        material.bind_to_shader(gl);
        
        gl.draw_elements_with_i32(
          WebGl2RenderingContext::TRIANGLES,
          group.index_count as i32,
          WebGl2RenderingContext::UNSIGNED_INT,
          group.start_index as i32 * 4, // 4 bytes per u32 index
        );
      }
    }
  }
}
```

### Memory Management

| Model Size | RAM Usage | VRAM Usage | Load Time | Strategy |
|------------|-----------|------------|-----------|----------|
| **< 10MB** | Low | Low | <1s | Direct loading |
| **10-50MB** | Medium | Medium | 1-5s | Streaming |
| **50-100MB** | High | High | 5-15s | LOD + streaming |
| **> 100MB** | Very High | Very High | >15s | Progressive loading |

## 🔍 Diagnostic Tools

### Model Analysis

```rust
// Built-in model diagnostic system
struct ModelDiagnostics {
  vertex_count: usize,
  triangle_count: usize,
  material_count: usize,
  texture_count: usize,
  memory_usage: usize,
  warnings: Vec<String>,
  errors: Vec<String>,
}

impl ModelDiagnostics {
  fn analyze_model(model: &ObjModel) -> Self {
    let mut diagnostics = Self::default();
    
    // Basic statistics
    diagnostics.vertex_count = model.vertices.len();
    diagnostics.triangle_count = model.faces.len();
    diagnostics.material_count = model.materials.len();
    
    // Memory usage estimation
    diagnostics.memory_usage = Self::estimate_memory_usage(model);
    
    // Check for common issues
    diagnostics.check_degenerate_faces(model);
    diagnostics.check_missing_normals(model);
    diagnostics.check_missing_textures(model);
    diagnostics.check_large_textures(model);
    
    diagnostics
  }
  
  fn check_degenerate_faces(&mut self, model: &ObjModel) {
    let mut degenerate_count = 0;
    
    for face in &model.faces {
      let v1 = model.vertices[face.vertex_indices[0]];
      let v2 = model.vertices[face.vertex_indices[1]];
      let v3 = model.vertices[face.vertex_indices[2]];
      
      // Check if triangle has zero area
      let edge1 = v2 - v1;
      let edge2 = v3 - v1;
      let cross = edge1.cross(edge2);
      
      if cross.length() < f32::EPSILON {
        degenerate_count += 1;
      }
    }
    
    if degenerate_count > 0 {
      self.warnings.push(
        format!("Found {} degenerate triangles", degenerate_count)
      );
    }
  }
  
  fn print_report(&self) {
    println!("=== Model Diagnostic Report ===");
    println!("Vertices: {}", self.vertex_count);
    println!("Triangles: {}", self.triangle_count);
    println!("Materials: {}", self.material_count);
    println!("Estimated Memory: {:.2} MB", self.memory_usage as f32 / (1024.0 * 1024.0));
    
    if !self.warnings.is_empty() {
      println!("\nWarnings:");
      for warning in &self.warnings {
        println!("  ⚠️  {}", warning);
      }
    }
    
    if !self.errors.is_empty() {
      println!("\nErrors:");
      for error in &self.errors {
        println!("  ❌ {}", error);
      }
    }
  }
}
```

## 🎯 Supported Features

### ✅ **Fully Supported**
- **OBJ Geometry** - Vertices, faces, normals, texture coordinates
- **MTL Materials** - All standard material properties
- **Texture Maps** - Diffuse, normal, specular, ambient occlusion
- **Multiple Objects** - Complex scenes with named objects
- **Large Models** - Optimized handling of high-polygon models

### ⚠️ **Partial Support**
- **Advanced Materials** - Some PBR extensions may not be supported
- **Complex Geometries** - NURBS and parametric surfaces not supported
- **Animations** - Static models only (no skeletal animation)

### ❌ **Not Supported**
- **OBJ Groups** - Group commands are ignored
- **Curve Objects** - Only polygonal geometry supported
- **Advanced Lighting** - Area lights, environment lighting limited

## 📚 Learning Resources

### PBR Theory
- **[PBR in OpenGL](https://learnopengl.com/PBR/Theory)** - Comprehensive PBR implementation guide
- **[Introduction to PBR](https://www.youtube.com/watch?v=gya7x9H3mV0&list=PLeb33PCuqDdesjTOgWXXAF4-gjknPPhBm&index=7)** - Video tutorial series
- **[Microfacet BRDF](https://simonstechblog.blogspot.com/2011/12/microfacet-brdf.html)** - Mathematical foundation

### 3D Graphics Programming
- **[Normal Mapping](https://learnopengl.com/Advanced-Lighting/Normal-Mapping)** - Surface detail enhancement
- **[Physically Based Rendering for Artists](https://www.youtube.com/watch?v=LNwMJeWFr0U)** - Artist-friendly PBR explanation
- **[Introduction to Physically Based Rendering](https://typhomnt.github.io/teaching/ray_tracing/pbr_intro/)** - Academic perspective

## 🛠️ Troubleshooting

### Common Issues and Solutions

#### Model Loading Problems
```rust
// Debug model loading issues
fn debug_model_loading(obj_path: &str) {
  match load_obj_model(obj_path) {
    Ok(model) => {
      let diagnostics = ModelDiagnostics::analyze_model(&model);
      diagnostics.print_report();
    },
    Err(e) => {
      println!("Failed to load model: {:?}", e);
      println!("Common fixes:");
      println!("  - Check file path is correct");
      println!("  - Ensure OBJ file is not corrupted");
      println!("  - Verify MTL file is in same directory");
      println!("  - Check texture paths in MTL file");
    }
  }
}
```

#### Performance Issues
- **Use `--release` flag** - Debug builds are 10-100x slower
- **Reduce model complexity** - Decimate high-polygon models
- **Optimize textures** - Use appropriate resolution and compression
- **Check diagnostics** - Look for degenerate geometry or missing data

#### Rendering Problems
- **Missing materials** - Check MTL file paths and material names
- **Black models** - Verify normal vectors are present and correct
- **Texture issues** - Ensure texture coordinates are within [0,1] range
- **Performance drops** - Monitor polygon count and texture memory usage

## 🤝 Contributing

Part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT