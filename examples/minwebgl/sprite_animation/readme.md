# 🎬 Sprite Animation System

> **Efficient 2D sprite animation using texture arrays and GPU acceleration**

A comprehensive sprite animation system demonstrating how to load sprite sheets, split them into individual frames, and play smooth animations using GPU texture arrays. Perfect for 2D games, UI animations, and educational content about efficient sprite rendering techniques.

![Sprite Animation Demo](./showcase.gif)

## ✨ Features

### 🎨 **Animation System**
- **Sprite Sheet Loading** - Import multi-frame animations from single images
- **Automatic Frame Extraction** - Split sprite sheets into individual frames
- **Smooth Playback** - Configurable frame rates and looping
- **GPU Texture Arrays** - Hardware-accelerated sprite storage and access

### 🔧 **Technical Implementation**
- **WebGL 2.0 Optimized** - Efficient GPU memory usage with texture arrays
- **Batch Rendering** - Minimal draw calls for multiple animated sprites
- **Flexible Layout** - Support for various sprite sheet configurations
- **Memory Efficient** - Single texture load for entire animation sequence

### 🎮 **Interactive Features**
- **Real-Time Animation** - Smooth frame transitions at any frame rate
- **Customizable Timing** - Adjustable animation speed and timing
- **Easy Integration** - Simple API for adding to games and applications

## 🚀 Quick Start

### Prerequisites
- WebGL 2.0 compatible browser
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Run the Example
```bash
# Navigate to sprite animation example
cd examples/minwebgl/sprite_animation

# Install trunk if needed
cargo install trunk

# Serve the example
trunk serve --release
```

Open http://localhost:8080 to see smooth sprite animation in action.

## 🎮 Using Custom Sprite Sheets

### Sprite Sheet Requirements
- **Grid Layout** - Frames arranged in regular rows and columns
- **Uniform Size** - Each frame must be the same dimensions
- **PNG/JPG Format** - Standard web image formats supported
- **Power of 2** - Recommended dimensions for optimal GPU performance

### Configuration
```rust
// Customize for your sprite sheet
let sprite_config = SpriteConfig {
  path: "static/your_sprite_sheet.png",
  sprites_in_row: 8,        // Frames per row
  sprite_width: 128,        // Individual frame width
  sprite_height: 128,       // Individual frame height
  total_frames: 64,         // Total number of frames
  frame_rate: 24.0,         // Animation speed (FPS)
};
```

### Setup Steps
1. **Add Sprite Sheet** - Place your sprite sheet in the `static/` folder
2. **Update Configuration** - Modify the sprite parameters in `main.rs`
3. **Build and Run** - `trunk serve --release` to see your animation

## 🔧 Technical Deep Dive

### Texture Array Implementation

Modern sprite animation uses GPU texture arrays for efficiency:

```rust
// Create texture array for sprite frames
fn create_sprite_texture_array(
  gl: &WebGl2RenderingContext,
  sprite_sheet: &Image,
  config: &SpriteConfig
) -> WebGlTexture {
  // Create 2D texture array
  let texture = gl.create_texture().unwrap();
  gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D_ARRAY, Some(&texture));
  
  // Allocate storage for all frames
  gl.tex_storage_3d(
    WebGl2RenderingContext::TEXTURE_2D_ARRAY,
    1, // mip levels
    WebGl2RenderingContext::RGBA8,
    config.sprite_width as i32,
    config.sprite_height as i32,
    config.total_frames as i32,
  );
  
  // Extract and upload each frame
  for frame_index in 0..config.total_frames {
    let frame_data = extract_frame(sprite_sheet, frame_index, config);
    
    gl.tex_sub_image_3d_with_u8_array(
      WebGl2RenderingContext::TEXTURE_2D_ARRAY,
      0, // mip level
      0, 0, frame_index as i32, // x, y, z offset
      config.sprite_width as i32,
      config.sprite_height as i32,
      1, // depth
      WebGl2RenderingContext::RGBA,
      WebGl2RenderingContext::UNSIGNED_BYTE,
      &frame_data,
    ).unwrap();
  }
  
  texture
}
```

### Frame Extraction Algorithm

```rust
// Extract individual frame from sprite sheet
fn extract_frame(
  sprite_sheet: &Image,
  frame_index: usize,
  config: &SpriteConfig
) -> Vec<u8> {
  let row = frame_index / config.sprites_in_row;
  let col = frame_index % config.sprites_in_row;
  
  let start_x = col * config.sprite_width;
  let start_y = row * config.sprite_height;
  
  let mut frame_data = Vec::new();
  
  // Copy pixel data for this frame
  for y in 0..config.sprite_height {
    for x in 0..config.sprite_width {
      let src_x = start_x + x;
      let src_y = start_y + y;
      
      let pixel_index = (src_y * sprite_sheet.width + src_x) * 4;
      let pixel = &sprite_sheet.data[pixel_index..pixel_index + 4];
      
      frame_data.extend_from_slice(pixel);
    }
  }
  
  frame_data
}
```

### Animation State Management

```rust
// Animation controller for smooth playback
struct SpriteAnimation {
  texture_array: WebGlTexture,
  current_frame: f32,
  frame_rate: f32,
  total_frames: usize,
  is_playing: bool,
  loop_animation: bool,
}

impl SpriteAnimation {
  fn update(&mut self, delta_time: f32) {
    if !self.is_playing { return; }
    
    // Advance animation
    self.current_frame += self.frame_rate * delta_time;
    
    // Handle looping
    if self.current_frame >= self.total_frames as f32 {
      if self.loop_animation {
        self.current_frame = 0.0;
      } else {
        self.current_frame = (self.total_frames - 1) as f32;
        self.is_playing = false;
      }
    }
  }
  
  fn get_current_frame_index(&self) -> i32 {
    self.current_frame.floor() as i32
  }
}
```

## 🎨 Shader Implementation

### Vertex Shader
```glsl
#version 300 es
precision mediump float;

// Attributes
in vec2 position;
in vec2 texCoord;

// Uniforms
uniform mat4 mvpMatrix;
uniform float currentFrame;

// Outputs
out vec2 vTexCoord;
out float vFrameIndex;

void main() {
  gl_Position = mvpMatrix * vec4(position, 0.0, 1.0);
  vTexCoord = texCoord;
  vFrameIndex = currentFrame;
}
```

### Fragment Shader
```glsl
#version 300 es
precision mediump float;

// Inputs
in vec2 vTexCoord;
in float vFrameIndex;

// Uniforms
uniform sampler2DArray spriteTexture;

// Output
out vec4 fragColor;

void main() {
  // Sample from texture array at current frame
  vec3 texCoord = vec3(vTexCoord, vFrameIndex);
  vec4 color = texture(spriteTexture, texCoord);
  
  // Discard transparent pixels
  if (color.a < 0.1) {
    discard;
  }
  
  fragColor = color;
}
```

## 📊 Performance Benefits

### Texture Array Advantages

| Method | GPU Memory | Draw Calls | Batch Efficiency |
|--------|------------|------------|------------------|
| **Individual Textures** | High | Many | Poor |
| **Sprite Sheets** | Medium | Medium | Good |
| **Texture Arrays** | Low | Few | Excellent |

### Performance Comparison
```rust
// Traditional method - multiple textures
for sprite in sprites {
  gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&sprite.texture));
  gl.draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 6); // Individual draw call
}

// Texture array method - batch rendering
gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D_ARRAY, Some(&sprite_array));
// Update frame indices in vertex buffer
update_frame_indices(&sprites);
gl.draw_arrays_instanced(
  WebGl2RenderingContext::TRIANGLES, 
  0, 6, 
  sprites.len() as i32 // All sprites in one draw call
);
```

## 🎯 Advanced Features

### Multi-Animation Support
```rust
// Manage multiple animations per sprite
struct AnimatedSprite {
  animations: HashMap<String, SpriteAnimation>,
  current_animation: String,
}

impl AnimatedSprite {
  fn play_animation(&mut self, name: &str) {
    if let Some(animation) = self.animations.get_mut(name) {
      self.current_animation = name.to_string();
      animation.is_playing = true;
      animation.current_frame = 0.0;
    }
  }
  
  fn update(&mut self, delta_time: f32) {
    if let Some(animation) = self.animations.get_mut(&self.current_animation) {
      animation.update(delta_time);
    }
  }
}
```

### Animation Events
```rust
// Trigger events at specific frames
struct AnimationEvent {
  frame: usize,
  callback: Box<dyn FnOnce()>,
}

struct EventDrivenAnimation {
  animation: SpriteAnimation,
  events: Vec<AnimationEvent>,
}

impl EventDrivenAnimation {
  fn update(&mut self, delta_time: f32) {
    let old_frame = self.animation.get_current_frame_index();
    self.animation.update(delta_time);
    let new_frame = self.animation.get_current_frame_index();
    
    // Check for frame transitions
    if old_frame != new_frame {
      self.process_frame_events(new_frame as usize);
    }
  }
}
```

### Interpolation and Blending
```glsl
// Smooth frame interpolation in shader
vec4 sampleInterpolated(sampler2DArray tex, vec2 uv, float frameFloat) {
  float frame1 = floor(frameFloat);
  float frame2 = ceil(frameFloat);
  float blend = fract(frameFloat);
  
  vec4 color1 = texture(tex, vec3(uv, frame1));
  vec4 color2 = texture(tex, vec3(uv, frame2));
  
  return mix(color1, color2, blend);
}
```

## 🎮 Common Use Cases

### Game Development
- **Character Animation** - Walking, running, attacking sequences
- **Environmental Effects** - Fire, water, particle animations
- **UI Elements** - Button hover states, loading indicators
- **Cutscene Assets** - Pre-rendered animation sequences

### Educational Applications
- **Algorithm Visualization** - Step-by-step process demonstration
- **Scientific Simulations** - Molecular movement, wave propagation
- **Interactive Tutorials** - Guided learning experiences
- **Art and Design** - Digital art creation and manipulation

## 🛠️ Optimization Tips

### Sprite Sheet Design
```rust
// Optimal sprite sheet configuration
struct OptimalConfig {
  // Keep power-of-2 dimensions when possible
  sprite_width: usize,    // 64, 128, 256, etc.
  sprite_height: usize,   // 64, 128, 256, etc.
  
  // Balance between memory and draw calls
  max_frames_per_sheet: usize, // 64-256 frames
  
  // Consider texture memory limits
  max_texture_size: usize, // 2048x2048 or 4096x4096
}
```

### Memory Management
```rust
// Preload common animations
struct AnimationCache {
  loaded_animations: HashMap<String, WebGlTexture>,
  memory_budget: usize,
}

impl AnimationCache {
  fn preload_critical_animations(&mut self) {
    // Load gameplay-essential animations first
    self.load_animation("player_walk");
    self.load_animation("player_jump");
    self.load_animation("enemy_idle");
  }
  
  fn unload_unused_animations(&mut self) {
    // Remove animations not used recently
    // Implement LRU cache eviction
  }
}
```

## 📚 Learning Resources

### Animation Theory
- **[Animation Principles](https://en.wikipedia.org/wiki/12_basic_principles_of_animation)** - Disney's fundamental animation principles
- **[Sprite Animation Techniques](https://2d-game-art-guru.com/2d-game-art-tutorials/sprite-animation-tutorial/)** - 2D game art techniques
- **[Texture Atlasing](https://en.wikipedia.org/wiki/Texture_atlas)** - Efficient texture management

### WebGL and Graphics
- **[WebGL2 Texture Arrays](https://webgl2fundamentals.org/webgl/lessons/webgl-2d-array-textures.html)** - Modern texture management
- **[GPU Instancing](https://webgl2fundamentals.org/webgl/lessons/webgl-instanced-drawing.html)** - Efficient sprite batching
- **[Game Programming Patterns](http://gameprogrammingpatterns.com/)** - Software engineering for games

## 🛠️ Troubleshooting

### Common Issues
- **Frame Timing** - Ensure consistent delta time for smooth animation
- **Texture Size Limits** - Check WebGL implementation limits
- **Alpha Blending** - Configure proper blending for transparent sprites
- **Memory Usage** - Monitor GPU memory consumption with large sprite sheets

### Debug Techniques
```rust
// Visualize current frame index
fn debug_render_frame_info(&self, animation: &SpriteAnimation) {
  println!("Current Frame: {:.2}", animation.current_frame);
  println!("Frame Rate: {:.1} FPS", animation.frame_rate);
  println!("Is Playing: {}", animation.is_playing);
}

// Visualize sprite sheet layout
fn debug_render_sprite_sheet(&self) {
  // Render entire sprite sheet with frame boundaries
  for i in 0..config.total_frames {
    self.render_frame_border(i);
  }
}
```

