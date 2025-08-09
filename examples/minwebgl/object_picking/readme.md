# 🎯 GPU Object Picking

> **Interactive 3D object selection using GPU-accelerated rendering techniques**

A comprehensive demonstration of object picking in 3D scenes using GPU-based color-coded rendering. Click on any 3D object to select it with pixel-perfect accuracy, showcasing modern interactive graphics programming techniques.

![Object Picking Demo](showcase.gif)

## ✨ Features

### 🎮 **Interactive Selection**
- **Pixel-Perfect Picking** - Click anywhere on an object to select it
- **Real-Time Feedback** - Instant visual response to mouse interaction
- **Multiple Objects** - Support for complex scenes with many selectable objects
- **GPU Acceleration** - Hardware-accelerated selection using render-to-texture

### 🔧 **Technical Implementation**
- **Color-Coded Rendering** - Unique color IDs for each object
- **Render-to-Texture** - Off-screen rendering for selection buffer
- **Pixel Reading** - GPU texture readback for mouse coordinate lookup
- **WebGL Integration** - Browser-native graphics API implementation

### 🎯 **Learning Objectives**
- **Render Target Management** - Multiple framebuffers and texture targets
- **GPU-CPU Communication** - Reading GPU data back to JavaScript
- **Interactive Graphics** - Mouse event handling in 3D space
- **Performance Optimization** - Efficient object identification techniques

## 🚀 Quick Start

### Prerequisites
- WebGL 2.0 compatible browser
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Run the Example
```bash
# Navigate to object picking example
cd examples/minwebgl/object_picking

# Install trunk if needed
cargo install trunk

# Serve the example
trunk serve --release
```

Open http://localhost:8080 and click on the 3D objects to see GPU picking in action.

## 🎮 How to Use

### Basic Interaction
1. **Mouse Movement** - Hover over objects to see cursor feedback
2. **Left Click** - Select objects by clicking on them
3. **Selection Feedback** - Selected objects change color or outline
4. **Multiple Selection** - Click different objects to switch selection

### Visual Feedback
- **Hover Effects** - Objects respond to mouse proximity
- **Selection Highlighting** - Clear indication of selected objects
- **Performance Metrics** - Real-time rendering statistics

## 🔧 Technical Deep Dive

### GPU-Based Object Picking

Traditional CPU-based ray-object intersection is expensive. GPU picking uses color-coded rendering:

```rust
// Assign unique color ID to each object
struct PickableObject
{
  mesh : Mesh,
  pick_id : u32,
  world_transform : Mat4,
}

// Convert ID to unique RGB color
fn id_to_color( id : u32 ) -> [ f32; 3 ]
{
  [
    ( ( id >> 16 ) & 0xFF ) as f32 / 255.0,
    ( ( id >> 8 ) & 0xFF ) as f32 / 255.0,
    ( id & 0xFF ) as f32 / 255.0,
  ]
}
```

### Render Pipeline

#### 1. Normal Scene Rendering
```glsl
// Fragment shader for visible scene
#version 300 es
precision mediump float;

uniform vec3 objectColor;
uniform vec3 lightPosition;

in vec3 fragPosition;
in vec3 fragNormal;
out vec4 fragColor;

void main() {
  // Standard Phong lighting calculation
  vec3 norm = normalize(fragNormal);
  vec3 lightDir = normalize(lightPosition - fragPosition);
  float diff = max(dot(norm, lightDir), 0.0);
  
  fragColor = vec4(objectColor * diff, 1.0);
}
```

#### 2. Pick Buffer Rendering
```glsl
// Fragment shader for picking buffer
#version 300 es
precision mediump float;

uniform vec3 pickColor; // Unique color for this object
out vec4 fragColor;

void main() {
  fragColor = vec4(pickColor, 1.0);
}
```

### Mouse to Object ID Conversion

```rust
// Read pixel from pick buffer at mouse coordinates
fn get_picked_object( mouse_x : i32, mouse_y : i32, gl : &WebGl2RenderingContext ) -> Option< u32 >
{
  // Bind pick framebuffer
  gl.bind_framebuffer( WebGl2RenderingContext::FRAMEBUFFER, Some( &pick_framebuffer ) );
  
  // Read single pixel at mouse position
  let mut pixel_data = [ 0u8; 4 ];
  gl.read_pixels
  (
    mouse_x, canvas_height - mouse_y, // Flip Y coordinate
    1, 1,
    WebGl2RenderingContext::RGBA,
    WebGl2RenderingContext::UNSIGNED_BYTE,
    Some( &mut pixel_data )
  );
  
  // Convert RGB back to object ID
  if pixel_data[ 0 ] == 0 && pixel_data[ 1 ] == 0 && pixel_data[ 2 ] == 0
  {
    None // Background pixel
  }
  else
  {
    let id = ( pixel_data[ 0 ] as u32 ) << 16 
           | ( pixel_data[ 1 ] as u32 ) << 8 
           | ( pixel_data[ 2 ] as u32 );
    Some( id )
  }
}
```

## 📊 Performance Considerations

### GPU Picking Advantages
- **Constant Time** - O(1) selection regardless of scene complexity
- **Pixel Perfect** - Exact object boundaries, no approximation
- **Hardware Accelerated** - Leverages GPU rasterization
- **Scalable** - Performance doesn't degrade with more objects

### Implementation Efficiency
```rust
// Batch render both buffers in single frame
struct DualPassRenderer
{
  scene_framebuffer : WebGlFramebuffer,
  pick_framebuffer : WebGlFramebuffer,
  scene_program : WebGlProgram,
  pick_program : WebGlProgram,
}

impl DualPassRenderer
{
  fn render_frame( &mut self, objects : &[ PickableObject ] )
  {
    // Pass 1: Render pick buffer (off-screen)
    self.render_pick_buffer( objects );
    
    // Pass 2: Render visible scene
    self.render_scene_buffer( objects );
    
    // Handle mouse picking if needed
    if let Some( mouse_pos ) = self.pending_mouse_click
    {
      let picked_id = self.read_pick_pixel( mouse_pos );
      self.handle_object_selection( picked_id );
    }
  }
}
```

### Memory Optimization
- **Texture Formats** - Use RGB8 for pick buffer (24-bit IDs)
- **Buffer Reuse** - Share depth buffer between passes
- **Selective Updates** - Only render pick buffer when needed

## 🎯 Use Cases and Applications

### Game Development
- **Character Selection** - Click to select units in RTS games
- **Inventory Systems** - Interactive item management interfaces
- **Building Placement** - Precise object positioning tools
- **Menu Interactions** - 3D UI element selection

### Professional Applications
- **CAD Software** - Engineering model part selection
- **Data Visualization** - Interactive chart and graph elements
- **Medical Imaging** - Anatomical structure selection
- **Architecture Tools** - Building component interaction

### Educational Examples
- **Interactive Demos** - Click-to-explore educational content
- **Algorithm Visualization** - Step-through interactive tutorials
- **3D Modeling** - Basic selection for modeling tools

## 🔗 Advanced Techniques

### Multi-Selection Support
```rust
// Hold Ctrl for multi-select
fn handle_mouse_click( &mut self, mouse_pos : ( i32, i32 ), ctrl_held : bool )
{
  let picked_id = self.read_pick_pixel( mouse_pos );
  
  if let Some( id ) = picked_id
  {
    if ctrl_held
    {
      // Add to selection set
      self.selected_objects.insert( id );
    }
    else
    {
      // Replace selection
      self.selected_objects.clear();
      self.selected_objects.insert( id );
    }
  }
}
```

### Hover Effects
```rust
// Track mouse movement for hover effects
fn handle_mouse_move( &mut self, mouse_pos : ( i32, i32 ) )
{
  let hovered_id = self.read_pick_pixel( mouse_pos );
  
  if hovered_id != self.current_hover
  {
    // Update hover state
    self.current_hover = hovered_id;
    self.request_redraw();
  }
}
```

### Selection Outlining
```glsl
// Edge detection for object outlining
float getEdgeStrength(vec2 uv) {
  vec2 texelSize = 1.0 / textureSize(pickBuffer, 0);
  
  vec3 center = texture(pickBuffer, uv).rgb;
  vec3 left = texture(pickBuffer, uv - vec2(texelSize.x, 0.0)).rgb;
  vec3 right = texture(pickBuffer, uv + vec2(texelSize.x, 0.0)).rgb;
  vec3 up = texture(pickBuffer, uv - vec2(0.0, texelSize.y)).rgb;
  vec3 down = texture(pickBuffer, uv + vec2(0.0, texelSize.y)).rgb;
  
  float edge = 0.0;
  edge += distance(center, left);
  edge += distance(center, right);
  edge += distance(center, up);
  edge += distance(center, down);
  
  return edge > 0.1 ? 1.0 : 0.0;
}
```

## 📚 Key Files Structure

```
examples/minwebgl/object_picking/
├── src/
│   ├── lib.rs              # Main application logic
│   ├── picking.rs          # GPU picking implementation
│   ├── scene.rs            # 3D scene management
│   └── objects.rs          # Pickable object definitions
├── shaders/
│   ├── scene.vert          # Scene rendering vertex shader
│   ├── scene.frag          # Scene rendering fragment shader
│   ├── pick.vert           # Pick buffer vertex shader
│   └── pick.frag           # Pick buffer fragment shader
└── assets/
    └── models/             # 3D object meshes
```

## 🛠️ Extending the Example

### Custom Object Types
```rust
// Add metadata to pickable objects
struct EnhancedPickableObject
{
  mesh : Mesh,
  pick_id : u32,
  name : String,
  object_type : ObjectType,
  properties : HashMap< String, Value >,
}

// Handle different object types
fn handle_object_selected( &mut self, id : u32 )
{
  if let Some( obj ) = self.objects.get( &id )
  {
    match obj.object_type
    {
      ObjectType::Weapon => self.show_weapon_info( obj ),
      ObjectType::Character => self.show_character_stats( obj ),
      ObjectType::Building => self.show_building_details( obj ),
    }
  }
}
```

### Performance Profiling
```rust
// Measure picking performance
struct PickingMetrics
{
  pick_buffer_render_time : f64,
  pixel_read_time : f64,
  total_objects : usize,
}

impl PickingMetrics
{
  fn measure_frame( &mut self )
  {
    let start = performance::now();
    // ... rendering code ...
    self.pick_buffer_render_time = performance::now() - start;
  }
}
```

## 📖 Learning Resources

- **[WebGL Object Picking](https://webglfundamentals.org/webgl/lessons/webgl-picking.html)** - Comprehensive picking tutorial
- **[GPU Gems - Selection Techniques](https://developer.nvidia.com/gpugems/gpugems2/part-iii-high-quality-rendering/chapter-22-hardware-occlusion-queries-made-useful)** - Advanced selection methods
- **[Real-Time Rendering](http://www.realtimerendering.com/)** - Interactive graphics theory
- **[OpenGL Superbible](https://www.openglsuperbible.com/)** - Low-level graphics programming

## 🛠️ Troubleshooting

### Common Issues
- **Incorrect Color Conversion** - Ensure proper ID to RGB mapping
- **Coordinate System Mismatch** - Remember to flip Y coordinates for WebGL
- **Framebuffer Binding** - Verify correct render target switching
- **Precision Loss** - Use adequate color bit depth for object count

### Debug Techniques
```rust
// Visualize pick buffer for debugging
fn debug_show_pick_buffer( &self )
{
  // Render pick buffer to screen instead of hidden framebuffer
  gl.bind_framebuffer( WebGl2RenderingContext::FRAMEBUFFER, None );
  // Render with pick shaders...
}
```

## 🤝 Contributing

Part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT
