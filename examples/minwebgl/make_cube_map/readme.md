# 💸 Dynamic Cube Map Generation

> **Real-time cube map rendering using multi-perspective camera system**

A comprehensive demonstration of dynamic cube map generation and display techniques using WebGL 2.0. This example showcases advanced rendering-to-texture methods, multi-pass rendering, and cube map sampling for realistic environmental reflections and advanced visual effects.

![Cube Map Example](./showcase.jpg)

## ✨ Features

### 🏏️ **Cube Map Generation**
- **6-Camera System** - Simultaneous rendering from all cube faces
- **Real-Time Updates** - Dynamic cube map refresh during runtime
- **Multi-Pass Rendering** - Efficient render-to-texture implementation
- **Custom Data Storage** - Normal and distance information in texture channels

### 🔧 **Technical Implementation**
- **WebGL Framebuffers** - Off-screen rendering to cube map faces
- **Perspective Projections** - Accurate 90-degree field-of-view cameras
- **Texture Cube Mapping** - Hardware-accelerated cube map sampling
- **Surface Analysis** - Normal vectors and distance field generation

### 🎮 **Visual Features**
- **Environmental Reflections** - Realistic surface reflections using cube maps
- **Surface Visualization** - RGB channels store surface normals
- **Distance Fields** - Alpha channel contains surface distance data
- **Interactive Display** - Real-time cube map visualization

## 🚀 Quick Start

### Prerequisites
- WebGL 2.0 compatible browser with cube map support
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Run the Example
```bash
# Navigate to cube map example
cd examples/minwebgl/make_cube_map

# Install trunk if needed
cargo install trunk

# Serve the example
trunk serve --release
```

Open http://localhost:8080 to explore dynamic cube map generation.

## 🔧 Technical Deep Dive

### Cube Map Generation Pipeline

The system renders to all six cube faces using multiple camera perspectives:

```rust
// Cube map generation system
struct CubeMapGenerator {
  framebuffer: WebGlFramebuffer,
  cube_texture: WebGlTexture,
  depth_buffer: WebGlRenderbuffer,
  cameras: [Camera; 6],
  resolution: u32,
}

impl CubeMapGenerator {
  fn new(gl: &WebGl2RenderingContext, resolution: u32) -> Result<Self, JsValue> {
    let cube_texture = Self::create_cube_texture(gl, resolution)?;
    let framebuffer = Self::create_framebuffer(gl)?;
    let depth_buffer = Self::create_depth_buffer(gl, resolution)?;
    let cameras = Self::create_cube_cameras();
    
    Ok(Self {
      framebuffer,
      cube_texture,
      depth_buffer,
      cameras,
      resolution,
    })
  }
}
```

### Cube Texture Creation

```rust
// Create cube map texture with all faces
fn create_cube_texture(
  gl: &WebGl2RenderingContext,
  size: u32
) -> Result<WebGlTexture, JsValue> {
  let texture = gl.create_texture().unwrap();
  
  gl.bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, Some(&texture));
  
  // Create storage for all 6 faces
  let faces = [
    WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X,
    WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X,
    WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y,
    WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y,
    WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z,
    WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z,
  ];
  
  for face in faces.iter() {
    gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
      *face,
      0, // mip level
      WebGl2RenderingContext::RGBA as i32,
      size as i32,
      size as i32,
      0, // border
      WebGl2RenderingContext::RGBA,
      WebGl2RenderingContext::UNSIGNED_BYTE,
      None,
    )?;
  }
  
  // Configure texture parameters
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_CUBE_MAP,
    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
    WebGl2RenderingContext::LINEAR as i32,
  );
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_CUBE_MAP,
    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
    WebGl2RenderingContext::LINEAR as i32,
  );
  
  Ok(texture)
}
```

### 6-Camera Setup

```rust
// Create cameras for all cube map faces
fn create_cube_cameras() -> [Camera; 6] {
  let center = Vec3::new(0.0, 0.0, 0.0);
  let fov = 90.0f32.to_radians(); // 90-degree FOV for cube faces
  let aspect = 1.0; // Square faces
  let near = 0.1;
  let far = 100.0;
  
  [
    // +X face (right)
    Camera::look_at(
      center, 
      center + Vec3::X, 
      -Vec3::Y,
      fov, aspect, near, far
    ),
    // -X face (left) 
    Camera::look_at(
      center, 
      center - Vec3::X, 
      -Vec3::Y,
      fov, aspect, near, far
    ),
    // +Y face (up)
    Camera::look_at(
      center, 
      center + Vec3::Y, 
      Vec3::Z,
      fov, aspect, near, far
    ),
    // -Y face (down)
    Camera::look_at(
      center, 
      center - Vec3::Y, 
      -Vec3::Z,
      fov, aspect, near, far
    ),
    // +Z face (forward)
    Camera::look_at(
      center, 
      center + Vec3::Z, 
      -Vec3::Y,
      fov, aspect, near, far
    ),
    // -Z face (backward)
    Camera::look_at(
      center, 
      center - Vec3::Z, 
      -Vec3::Y,
      fov, aspect, near, far
    ),
  ]
}
```

### Multi-Pass Rendering

```rust
// Render scene to all cube faces
impl CubeMapGenerator {
  fn generate_cube_map(&self, gl: &WebGl2RenderingContext, scene: &Scene) {
    // Bind framebuffer for off-screen rendering
    gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.framebuffer));
    gl.viewport(0, 0, self.resolution as i32, self.resolution as i32);
    
    let faces = [
      WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_X,
      WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_X,
      WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Y,
      WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Y,
      WebGl2RenderingContext::TEXTURE_CUBE_MAP_POSITIVE_Z,
      WebGl2RenderingContext::TEXTURE_CUBE_MAP_NEGATIVE_Z,
    ];
    
    // Render to each face
    for (i, face) in faces.iter().enumerate() {
      // Attach current face to framebuffer
      gl.framebuffer_texture_2d(
        WebGl2RenderingContext::FRAMEBUFFER,
        WebGl2RenderingContext::COLOR_ATTACHMENT0,
        *face,
        Some(&self.cube_texture),
        0, // mip level
      );
      
      // Clear and render scene from this camera
      gl.clear(
        WebGl2RenderingContext::COLOR_BUFFER_BIT | 
        WebGl2RenderingContext::DEPTH_BUFFER_BIT
      );
      
      scene.render_with_camera(&self.cameras[i]);
    }
    
    // Restore default framebuffer
    gl.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);
  }
}
```

### Surface Data Encoding

The cube map stores specialized data in different channels:

```glsl
// Fragment shader for cube map generation
#version 300 es
precision mediump float;

in vec3 worldPosition;
in vec3 worldNormal;
in vec3 cameraPosition;

out vec4 fragColor;

void main() {
  // Normalize surface normal
  vec3 normal = normalize(worldNormal);
  
  // Calculate distance to surface from camera
  float distance = length(worldPosition - cameraPosition);
  
  // Encode surface normal in RGB channels (range [-1,1] to [0,1])
  vec3 encodedNormal = normal * 0.5 + 0.5;
  
  // Store distance in alpha channel (normalized)
  float normalizedDistance = distance / 100.0; // Assuming max distance of 100
  
  // Final cube map texel
  fragColor = vec4(encodedNormal, normalizedDistance);
}
```

### Cube Map Sampling

```glsl
// Sample cube map for reflections
#version 300 es
precision mediump float;

in vec3 worldPosition;
in vec3 worldNormal;
in vec3 cameraPosition;

uniform samplerCube cubeMap;
uniform float reflectivity;

out vec4 fragColor;

vec3 sampleCubeMap(vec3 direction) {
  vec4 cubeData = texture(cubeMap, direction);
  
  // Decode normal from RGB channels
  vec3 surfaceNormal = cubeData.rgb * 2.0 - 1.0;
  
  // Get distance from alpha channel
  float surfaceDistance = cubeData.a * 100.0;
  
  return cubeData.rgb; // Use as color for now
}

void main() {
  vec3 viewDirection = normalize(worldPosition - cameraPosition);
  vec3 reflectionDir = reflect(viewDirection, normalize(worldNormal));
  
  // Sample cube map
  vec3 reflectionColor = sampleCubeMap(reflectionDir);
  
  // Apply reflectivity
  vec3 baseColor = vec3(0.2, 0.3, 0.8); // Base material color
  vec3 finalColor = mix(baseColor, reflectionColor, reflectivity);
  
  fragColor = vec4(finalColor, 1.0);
}
```

## 🎨 Diamond Object Analysis

### Surface Normal Visualization

The example renders a diamond object from the inside, encoding surface properties:

```rust
// Diamond geometry with detailed surface normals
struct DiamondObject {
  vertices: Vec<Vertex>,
  normals: Vec<Vec3>,
  faces: Vec<Face>,
}

impl DiamondObject {
  fn generate_diamond() -> Self {
    // Generate diamond facets with precise normal vectors
    let mut vertices = Vec::new();
    let mut normals = Vec::new();
    
    // Top point
    vertices.push(Vertex::new(0.0, 1.0, 0.0));
    // Bottom point  
    vertices.push(Vertex::new(0.0, -1.0, 0.0));
    
    // Generate ring of vertices around middle
    let ring_count = 8;
    for i in 0..ring_count {
      let angle = 2.0 * PI * i as f32 / ring_count as f32;
      let x = angle.cos();
      let z = angle.sin();
      vertices.push(Vertex::new(x, 0.0, z));
    }
    
    // Calculate face normals for diamond facets
    // ... generate faces and compute normals
    
    Self { vertices, normals, faces }
  }
}
```

### Distance Field Generation

```glsl
// Calculate distance field in fragment shader
float calculateDistanceToSurface(vec3 position, vec3 surfacePoint) {
  return length(position - surfacePoint);
}

void main() {
  vec3 surfacePoint = worldPosition;
  vec3 cameraPos = cameraPosition;
  
  // Calculate distance field
  float distance = calculateDistanceToSurface(cameraPos, surfacePoint);
  
  // Encode in alpha channel with appropriate range
  float normalizedDistance = clamp(distance / maxDistance, 0.0, 1.0);
  
  fragColor = vec4(encodedNormal, normalizedDistance);
}
```

## 📊 Performance Considerations

### Cube Map Resolution Impact

| Resolution | Memory Usage | Performance | Quality |
|------------|--------------|-------------|----------|
| **256x256** | 1.5MB | Excellent | Good |
| **512x512** | 6MB | Good | Very Good |
| **1024x1024** | 24MB | Fair | Excellent |
| **2048x2048** | 96MB | Poor | Outstanding |

### Optimization Strategies

```rust
// Level-of-detail for cube map generation
struct CubeMapLOD {
  base_resolution: u32,
  distance_thresholds: Vec<f32>,
  lod_scales: Vec<f32>,
}

impl CubeMapLOD {
  fn get_resolution(&self, distance_to_camera: f32) -> u32 {
    for (i, threshold) in self.distance_thresholds.iter().enumerate() {
      if distance_to_camera < *threshold {
        return (self.base_resolution as f32 * self.lod_scales[i]) as u32;
      }
    }
    
    // Farthest LOD
    (self.base_resolution as f32 * self.lod_scales.last().unwrap()) as u32
  }
}

// Selective face updates
struct SelectiveCubeMapUpdater {
  face_priorities: [f32; 6],
  update_budget: usize, // Max faces to update per frame
}

impl SelectiveCubeMapUpdater {
  fn update_cube_map(&mut self, camera: &Camera) {
    // Calculate face importance based on camera direction
    self.calculate_face_priorities(camera);
    
    // Sort faces by priority
    let mut face_indices: Vec<usize> = (0..6).collect();
    face_indices.sort_by(|&a, &b| {
      self.face_priorities[b].partial_cmp(&self.face_priorities[a]).unwrap()
    });
    
    // Update only the most important faces
    for i in 0..self.update_budget.min(6) {
      let face_index = face_indices[i];
      self.update_face(face_index);
    }
  }
}
```

## 🎯 Advanced Applications

### Environmental Lighting

```glsl
// Use cube map for image-based lighting
vec3 sampleEnvironmentLighting(vec3 normal, samplerCube envMap) {
  // Sample diffuse irradiance from cube map
  vec3 irradiance = texture(envMap, normal).rgb;
  
  // Apply cosine-weighted hemisphere sampling
  return irradiance * PI;
}

// Specular reflections with roughness
vec3 sampleSpecularReflection(vec3 reflectionDir, float roughness, samplerCube envMap) {
  // Calculate mip level based on roughness
  float mipLevel = roughness * float(textureQueryLevels(envMap) - 1);
  
  // Sample pre-filtered environment map
  return textureLod(envMap, reflectionDir, mipLevel).rgb;
}
```

### Dynamic Reflections

```rust
// Real-time reflection updates
struct DynamicReflectionSystem {
  cube_generators: Vec<CubeMapGenerator>,
  reflection_objects: Vec<ReflectiveObject>,
  update_frequency: f32,
  last_update: f32,
}

impl DynamicReflectionSystem {
  fn update(&mut self, current_time: f32, scene: &Scene) {
    if current_time - self.last_update >= self.update_frequency {
      // Update cube maps for dynamic objects
      for (generator, object) in self.cube_generators.iter().zip(&self.reflection_objects) {
        if object.needs_update(current_time) {
          generator.generate_cube_map(scene, object.position);
        }
      }
      
      self.last_update = current_time;
    }
  }
}
```

## 📚 Learning Resources

### Cube Mapping Theory
- **[Cube Environment Mapping](https://en.wikipedia.org/wiki/Cube_mapping)** - Mathematical foundation
- **[OpenGL Cube Maps](https://learnopengl.com/Advanced-OpenGL/Cubemaps)** - Implementation techniques
- **[Environment Mapping](https://developer.nvidia.com/gpugems/gpugems/part-i-natural-effects/chapter-7-environment-mapping-techniques)** - Advanced applications

### Real-Time Rendering
- **[Real-Time Rendering](http://www.realtimerendering.com/)** - Comprehensive graphics theory
- **[GPU Gems](https://developer.nvidia.com/gpugems)** - Advanced GPU programming
- **[Physically Based Rendering](https://pbr-book.org/)** - Modern rendering techniques

## 🛠️ Troubleshooting

### Common Issues
- **Seam Artifacts** - Ensure proper face orientation and filtering
- **Performance Problems** - Consider reducing cube map resolution
- **Memory Usage** - Implement level-of-detail systems
- **Precision Issues** - Use appropriate texture formats for data storage

### Debug Techniques
```rust
// Visualize cube map faces
fn debug_render_cube_faces(gl: &WebGl2RenderingContext, cube_texture: &WebGlTexture) {
  let face_names = ["PX", "NX", "PY", "NY", "PZ", "NZ"];
  
  for (i, face) in face_names.iter().enumerate() {
    // Render each face to a debug quad
    render_debug_quad(gl, cube_texture, i, face);
  }
}

// Check cube map completeness
fn validate_cube_map(gl: &WebGl2RenderingContext, texture: &WebGlTexture) -> bool {
  gl.bind_texture(WebGl2RenderingContext::TEXTURE_CUBE_MAP, Some(texture));
  
  // Check if cube map is complete
  let status = gl.check_framebuffer_status(WebGl2RenderingContext::FRAMEBUFFER);
  status == WebGl2RenderingContext::FRAMEBUFFER_COMPLETE
}
```

## 🤝 Contributing

Part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT