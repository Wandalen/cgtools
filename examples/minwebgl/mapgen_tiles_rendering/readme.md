# 🏺 Procedural Tile Map Rendering

> **Efficient GPU-based tile rendering system using WebGL 2.0 and texture arrays**

A comprehensive tile-based rendering system demonstrating advanced WebGL techniques for game development and procedural content generation. This example showcases efficient tile map rendering using texture arrays, unsigned integer textures, and optimized GPU memory management.

![Tile Rendering Demo](./showcase.png)

## ✨ Features

### 🎮 **Tile Rendering System**
- **Texture Array Rendering** - Efficient multi-tile rendering in single draw call
- **GPU-Optimized** - Hardware-accelerated tile placement and sampling
- **Flexible Tile Maps** - Support for various tile map configurations
- **Memory Efficient** - Optimized texture storage and access patterns

### 🔧 **Technical Implementation**
- **WebGL 2.0 Features** - Unsigned integer textures and advanced sampling
- **Vertex Array Objects** - Efficient geometry management
- **Multi-Texture Binding** - Simultaneous tileset and tilemap texture usage
- **Custom Shaders** - Specialized tile rendering fragment shaders

### 🎨 **Customization Options**
- **Custom Tilesets** - Easy integration of user-provided tile images
- **Configurable Maps** - Flexible tile map data structure
- **Scalable Architecture** - Support for various tile sizes and counts

## 🚀 Quick Start

### Prerequisites
- WebGL 2.0 compatible browser
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Run the Example
```bash
# Navigate to tile rendering example
cd examples/minwebgl/mapgen_tiles_rendering

# Install trunk if needed
cargo install trunk

# Serve the example
trunk serve --release
```

Open http://localhost:8080 to see tile-based rendering in action.

## 🔧 Technical Deep Dive

### Tile Rendering Architecture

The system uses a multi-texture approach with texture arrays for efficient rendering:

```rust
// Tile rendering system structure
struct TileRenderer {
  tileset_texture: WebGlTexture,      // Contains all tile images
  tilemap_texture: WebGlTexture,      // Contains tile ID data
  vertex_array: WebGlVertexArrayObject,
  shader_program: WebGlProgram,
  tile_count: i32,
  map_dimensions: (usize, usize),
}

impl TileRenderer {
  fn new(gl: &WebGl2RenderingContext) -> Result<Self, JsValue> {
    let tileset_texture = Self::create_tileset_texture(gl)?;
    let tilemap_texture = Self::create_tilemap_texture(gl)?;
    let vertex_array = Self::setup_quad_geometry(gl)?;
    let shader_program = Self::create_tile_shader_program(gl)?;
    
    Ok(Self {
      tileset_texture,
      tilemap_texture,
      vertex_array,
      shader_program,
      tile_count: TILE_COUNT,
      map_dimensions: (MAP_WIDTH, MAP_HEIGHT),
    })
  }
}
```

### Texture Array Creation

```rust
// Create tileset texture from image
fn create_tileset_texture(gl: &WebGl2RenderingContext) -> Result<WebGlTexture, JsValue> {
  let texture = gl.create_texture().unwrap();
  
  // Bind texture for configuration
  gl.active_texture(WebGl2RenderingContext::TEXTURE0);
  gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
  
  // Configure texture parameters for tile rendering
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_2D,
    WebGl2RenderingContext::TEXTURE_WRAP_S,
    WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
  );
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_2D,
    WebGl2RenderingContext::TEXTURE_WRAP_T,
    WebGl2RenderingContext::CLAMP_TO_EDGE as i32,
  );
  
  // Use nearest filtering for pixel-perfect tiles
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_2D,
    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
    WebGl2RenderingContext::NEAREST as i32,
  );
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_2D,
    WebGl2RenderingContext::TEXTURE_MAG_FILTER,
    WebGl2RenderingContext::NEAREST as i32,
  );
  
  Ok(texture)
}
```

### Tilemap Data Texture

```rust
// Create tilemap texture from raw tile ID data
fn create_tilemap_texture(
  gl: &WebGl2RenderingContext,
  tile_data: &[u8],
  width: usize,
  height: usize
) -> Result<WebGlTexture, JsValue> {
  let texture = gl.create_texture().unwrap();
  
  gl.active_texture(WebGl2RenderingContext::TEXTURE1);
  gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
  
  // Upload raw tile ID data
  gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
    WebGl2RenderingContext::TEXTURE_2D,
    0, // mip level
    WebGl2RenderingContext::R8UI as i32, // Unsigned integer format
    width as i32,
    height as i32,
    0, // border
    WebGl2RenderingContext::RED_INTEGER,
    WebGl2RenderingContext::UNSIGNED_BYTE,
    Some(tile_data),
  )?;
  
  // No filtering for integer textures
  gl.tex_parameteri(
    WebGl2RenderingContext::TEXTURE_2D,
    WebGl2RenderingContext::TEXTURE_MIN_FILTER,
    WebGl2RenderingContext::NEAREST as i32,
  );
  
  Ok(texture)
}
```

### Tile Shader Implementation

#### Vertex Shader
```glsl
#version 300 es
precision mediump float;

// Quad vertex attributes
in vec2 position;
in vec2 texCoord;

// Transformation matrix
uniform mat4 mvpMatrix;

// Pass texture coordinates to fragment shader
out vec2 vTexCoord;

void main() {
  gl_Position = mvpMatrix * vec4(position, 0.0, 1.0);
  vTexCoord = texCoord;
}
```

#### Fragment Shader
```glsl
#version 300 es
precision mediump float;

in vec2 vTexCoord;

// Texture samplers
uniform sampler2D tilesetTexture;     // Contains tile images
uniform usampler2D tilemapTexture;    // Contains tile IDs (unsigned integer)

// Tile configuration
uniform float tileCount;              // Number of tiles in tileset
uniform vec2 tilemapSize;             // Tilemap dimensions
uniform vec2 tileSize;                // Individual tile size in tileset

out vec4 fragColor;

void main() {
  // Calculate which tile we're rendering
  vec2 tileCoord = floor(vTexCoord * tilemapSize) / tilemapSize;
  
  // Sample tile ID from tilemap texture
  uint tileId = texture(tilemapTexture, tileCoord).r;
  
  // Calculate UV coordinates within the tile
  vec2 localUV = fract(vTexCoord * tilemapSize);
  
  // Calculate tileset coordinates
  float tileY = float(tileId) / tileCount;
  vec2 tilesetUV = vec2(localUV.x, tileY + localUV.y / tileCount);
  
  // Sample from tileset texture
  vec4 tileColor = texture(tilesetTexture, tilesetUV);
  
  fragColor = tileColor;
}
```

### Advanced Tile Rendering Techniques

#### Multi-Layer Tile Rendering

```rust
// Support for multiple tile layers
struct MultiLayerTileRenderer {
  layers: Vec<TileLayer>,
  blend_modes: Vec<BlendMode>,
}

struct TileLayer {
  tilemap_texture: WebGlTexture,
  tileset_texture: WebGlTexture,
  opacity: f32,
  visible: bool,
}

enum BlendMode {
  Normal,
  Multiply,
  Screen,
  Overlay,
}

impl MultiLayerTileRenderer {
  fn render_layers(&self, gl: &WebGl2RenderingContext) {
    // Enable blending for layer composition
    gl.enable(WebGl2RenderingContext::BLEND);
    
    for (layer, blend_mode) in self.layers.iter().zip(&self.blend_modes) {
      if !layer.visible { continue; }
      
      // Set blend mode
      match blend_mode {
        BlendMode::Normal => {
          gl.blend_func(
            WebGl2RenderingContext::SRC_ALPHA,
            WebGl2RenderingContext::ONE_MINUS_SRC_ALPHA
          );
        },
        BlendMode::Multiply => {
          gl.blend_func(
            WebGl2RenderingContext::DST_COLOR,
            WebGl2RenderingContext::ZERO
          );
        },
        // ... other blend modes
      }
      
      self.render_single_layer(gl, layer);
    }
  }
}
```

## 🎮 Customization Guide

### Creating Custom Tilesets

1. **Tileset Image Requirements**
   - **Vertical Layout** - Tiles arranged top to bottom
   - **Equal Dimensions** - Each tile must be the same size
   - **No Padding** - Tiles should be tightly packed
   - **Power of 2** - Recommended for optimal GPU performance

2. **Configuration Constants**
   ```rust
   // Modify these constants for your tileset
   const TILE_COUNT: i32 = 8;           // Number of tiles in your tileset
   const TILE_WIDTH: u32 = 64;          // Width of each tile in pixels
   const TILE_HEIGHT: u32 = 64;         // Height of each tile in pixels
   const TILESET_PATH: &str = "static/custom_tileset.png";
   ```

3. **Tilemap Data Format**
   ```rust
   // Define your tile map layout
   const MAP_WIDTH: usize = 8;
   const MAP_HEIGHT: usize = 8;
   
   const TILE_MAP_DATA: [u8; 64] = [
     0, 0, 1, 1, 2, 2, 3, 3,
     0, 1, 1, 2, 2, 3, 3, 4,
     1, 1, 2, 2, 3, 3, 4, 4,
     1, 2, 2, 3, 3, 4, 4, 5,
     2, 2, 3, 3, 4, 4, 5, 5,
     2, 3, 3, 4, 4, 5, 5, 6,
     3, 3, 4, 4, 5, 5, 6, 6,
     3, 4, 4, 5, 5, 6, 6, 7,
   ];
   ```

### Dynamic Tile Map Generation

```rust
// Procedural tile map generation
struct ProceduralTileMapGenerator {
  noise: PerlinNoise,
  tile_rules: HashMap<u8, TileRule>,
}

struct TileRule {
  elevation_range: (f32, f32),
  moisture_range: (f32, f32),
  tile_id: u8,
}

impl ProceduralTileMapGenerator {
  fn generate_map(&self, width: usize, height: usize) -> Vec<u8> {
    let mut map_data = Vec::with_capacity(width * height);
    
    for y in 0..height {
      for x in 0..width {
        let elevation = self.noise.sample(x as f32 * 0.01, y as f32 * 0.01, 0.0);
        let moisture = self.noise.sample(x as f32 * 0.005, y as f32 * 0.005, 100.0);
        
        let tile_id = self.determine_tile(elevation, moisture);
        map_data.push(tile_id);
      }
    }
    
    map_data
  }
  
  fn determine_tile(&self, elevation: f32, moisture: f32) -> u8 {
    for (tile_id, rule) in &self.tile_rules {
      if elevation >= rule.elevation_range.0 && elevation <= rule.elevation_range.1 &&
         moisture >= rule.moisture_range.0 && moisture <= rule.moisture_range.1 {
        return *tile_id;
      }
    }
    0 // Default tile
  }
}
```

## 📊 Performance Optimization

### Texture Memory Management

| Technique | Memory Usage | Performance | Use Case |
|-----------|--------------|-------------|----------|
| **Single Texture** | High | Poor | Small maps |
| **Texture Atlas** | Medium | Good | Medium maps |
| **Texture Array** | Low | Excellent | Large maps |
| **Streaming** | Very Low | Variable | Infinite maps |

### Efficient Rendering Pipeline

```rust
// Optimized rendering pipeline
struct OptimizedTileRenderer {
  instanced_renderer: InstancedRenderer,
  frustum_culler: FrustumCuller,
  batch_manager: TileBatchManager,
}

impl OptimizedTileRenderer {
  fn render_optimized(&mut self, camera: &Camera) {
    // 1. Frustum culling - only render visible tiles
    let visible_tiles = self.frustum_culler.cull_tiles(camera, &self.tile_grid);
    
    // 2. Batch similar tiles together
    let batches = self.batch_manager.create_batches(&visible_tiles);
    
    // 3. Instanced rendering for identical tiles
    for batch in batches {
      self.instanced_renderer.render_batch(&batch);
    }
  }
}

// Instanced tile rendering
struct InstancedTileBatch {
  tile_id: u8,
  positions: Vec<Vec2>,
  instance_buffer: WebGlBuffer,
}

impl InstancedTileBatch {
  fn render(&self, gl: &WebGl2RenderingContext, program: &WebGlProgram) {
    // Bind instance data
    gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.instance_buffer));
    
    // Draw all instances in one call
    gl.draw_arrays_instanced(
      WebGl2RenderingContext::TRIANGLES,
      0,
      6, // Quad vertices
      self.positions.len() as i32,
    );
  }
}
```

## 🎯 Advanced Features

### Tile Animation System

```glsl
// Animated tiles in fragment shader
uniform float time;
uniform float animationSpeed;

void main() {
  vec2 tileCoord = floor(vTexCoord * tilemapSize) / tilemapSize;
  uint baseTileId = texture(tilemapTexture, tileCoord).r;
  
  // Animated tile logic
  uint animatedTileId;
  if (baseTileId >= 64u && baseTileId < 68u) { // Water tiles
    float animFrame = mod(time * animationSpeed, 4.0);
    animatedTileId = baseTileId + uint(animFrame);
  } else {
    animatedTileId = baseTileId;
  }
  
  // Sample animated tile
  vec2 localUV = fract(vTexCoord * tilemapSize);
  float tileY = float(animatedTileId) / tileCount;
  vec2 tilesetUV = vec2(localUV.x, tileY + localUV.y / tileCount);
  
  fragColor = texture(tilesetTexture, tilesetUV);
}
```

### Collision Detection Integration

```rust
// Tile-based collision system
struct TileCollisionMap {
  collision_data: Vec<Vec<bool>>,
  tile_size: f32,
}

impl TileCollisionMap {
  fn check_collision(&self, world_pos: Vec2) -> bool {
    let tile_x = (world_pos.x / self.tile_size).floor() as usize;
    let tile_y = (world_pos.y / self.tile_size).floor() as usize;
    
    if tile_y < self.collision_data.len() && tile_x < self.collision_data[tile_y].len() {
      self.collision_data[tile_y][tile_x]
    } else {
      false
    }
  }
  
  fn get_collision_rect(&self, tile_x: usize, tile_y: usize) -> Option<Rect> {
    if self.collision_data[tile_y][tile_x] {
      Some(Rect::new(
        tile_x as f32 * self.tile_size,
        tile_y as f32 * self.tile_size,
        self.tile_size,
        self.tile_size,
      ))
    } else {
      None
    }
  }
}
```

## 📚 Learning Resources

### Tile-Based Game Development
- **[Tile-Based Games](https://gamedevelopment.tutsplus.com/tutorials/an-introduction-to-creating-a-tile-based-puzzle-game--gamedev-14041)** - Comprehensive guide
- **[2D Game Art](https://opengameart.org/)** - Free tile sets and sprites
- **[Tiled Map Editor](https://www.mapeditor.org/)** - Professional tile map editing tool

### Advanced Rendering
- **[WebGL2 Fundamentals](https://webgl2fundamentals.org/)** - WebGL 2.0 techniques
- **[GPU Gems](https://developer.nvidia.com/gpugems)** - Advanced graphics programming
- **[Real-Time Rendering](http://www.realtimerendering.com/)** - Graphics theory and practice

## 🛠️ Troubleshooting

### Common Issues
- **Texture Filtering** - Use `NEAREST` for pixel-perfect tiles
- **Integer Textures** - Remember to use `usampler2D` for tile ID textures
- **Tile Bleeding** - Ensure proper UV coordinate clamping
- **Performance** - Consider frustum culling for large maps

### Debug Techniques
```rust
// Debug tile rendering
fn debug_render_tile_grid(gl: &WebGl2RenderingContext) {
  // Render tile boundaries for debugging
  gl.polygon_mode(WebGl2RenderingContext::FRONT_AND_BACK, WebGl2RenderingContext::LINE);
  // ... render wireframe grid ...
  gl.polygon_mode(WebGl2RenderingContext::FRONT_AND_BACK, WebGl2RenderingContext::FILL);
}

// Visualize tile IDs
fn debug_tilemap_texture(tilemap_data: &[u8], width: usize, height: usize) {
  for y in 0..height {
    for x in 0..width {
      print!("{:2} ", tilemap_data[y * width + x]);
    }
    println!();
  }
}
```

