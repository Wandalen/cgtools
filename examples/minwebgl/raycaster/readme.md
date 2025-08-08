# 🕳️ Raycasting Engine

> **Classic 2.5D rendering technique powering iconic games like Wolfenstein 3D**

A faithful implementation of the DDA (Digital Differential Analyzer) raycasting algorithm that revolutionized early 3D gaming. Experience smooth pseudo-3D graphics with real-time wall rendering, player movement, and classic FPS-style navigation.

![Raycasting Demo](showcase.png)

## ✨ Features

### 🎮 **Classic Gameplay**
- **Wolfenstein-Style Movement** - WASD navigation with smooth turning
- **Real-Time Rendering** - 60fps raycasting with adjustable quality
- **Wall Detection** - Precise collision detection and wall rendering
- **Perspective Projection** - Realistic depth and height scaling

### 🔧 **Technical Implementation**
- **DDA Algorithm** - Efficient grid traversal for ray-wall intersection
- **Texture Mapping** - Wall textures with proper UV coordinates
- **Distance Calculation** - Accurate depth for realistic perspective
- **Performance Optimization** - Configurable ray count for quality/speed balance

### 📚 **Educational Value**
- **Algorithm Visualization** - See raycasting principles in action
- **Graphics Programming** - Learn fundamental 3D rendering concepts
- **Game History** - Experience technology that shaped gaming
- **Mathematical Foundation** - Vector mathematics and grid traversal

## 🚀 Quick Start

### Prerequisites
- WebGL 2.0 compatible browser
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Run the Example
```bash
# Navigate to raycasting example
cd examples/minwebgl/raycaster

# Install trunk if needed
cargo install trunk

# Serve the example
trunk serve --release
```

Open http://localhost:8080 and use WASD keys to explore the classic raycasted world.

## 🎮 Controls

### Player Movement
- **W** - Move forward
- **S** - Move backward  
- **A** - Turn left
- **D** - Turn right

### Debug Options
- **Mouse** - Look around (if mouse capture enabled)
- **Arrow Keys** - Alternative turning controls
- **+/-** - Adjust movement speed (if implemented)

## 🔧 Technical Deep Dive

### Raycasting Fundamentals

Raycasting creates a 3D perspective from a 2D map by casting rays from the player's position:

```rust
// Cast rays across the screen width
for x in 0..screen_width {
  // Calculate ray direction
  let camera_x = 2.0 * x as f32 / screen_width as f32 - 1.0;
  let ray_dir = player_dir + camera_plane * camera_x;
  
  // Perform DDA to find wall intersection
  let hit = cast_ray(player_pos, ray_dir, &world_map);
  
  // Calculate wall height based on distance
  let wall_height = screen_height as f32 / hit.distance;
  
  // Draw vertical line for this ray
  draw_wall_slice(x, wall_height, hit.texture, hit.tex_x);
}
```

### DDA Algorithm Implementation

The **Digital Differential Analyzer** efficiently traverses the grid:

```rust
fn cast_ray(start: Vec2, direction: Vec2, map: &WorldMap) -> RayHit {
  // Which grid cell we're in
  let mut map_x = start.x as i32;
  let mut map_y = start.y as i32;
  
  // Calculate delta distances
  let delta_dist_x = (1.0 / direction.x).abs();
  let delta_dist_y = (1.0 / direction.y).abs();
  
  // Calculate step direction and initial side_dist
  let (step_x, mut side_dist_x) = if direction.x < 0.0 {
    (-1, (start.x - map_x as f32) * delta_dist_x)
  } else {
    (1, (map_x as f32 + 1.0 - start.x) * delta_dist_x)
  };
  
  let (step_y, mut side_dist_y) = if direction.y < 0.0 {
    (-1, (start.y - map_y as f32) * delta_dist_y)
  } else {
    (1, (map_y as f32 + 1.0 - start.y) * delta_dist_y)
  };
  
  // Perform DDA
  loop {
    // Jump to next map square, either in x-direction, or in y-direction
    if side_dist_x < side_dist_y {
      side_dist_x += delta_dist_x;
      map_x += step_x;
      side = 0; // X-side
    } else {
      side_dist_y += delta_dist_y;
      map_y += step_y;
      side = 1; // Y-side
    }
    
    // Check if ray has hit a wall
    if map.get(map_x, map_y).is_wall() {
      break;
    }
  }
  
  // Calculate distance and texture coordinates
  let perpendicular_wall_dist = if side == 0 {
    (map_x as f32 - start.x + (1 - step_x) as f32 / 2.0) / direction.x
  } else {
    (map_y as f32 - start.y + (1 - step_y) as f32 / 2.0) / direction.y
  };
  
  RayHit {
    distance: perpendicular_wall_dist,
    wall_type: map.get(map_x, map_y),
    side,
    texture_x: calculate_texture_x(start, direction, perpendicular_wall_dist, side),
  }
}
```

### Wall Rendering

Each ray determines the height of a vertical strip:

```rust
fn draw_wall_slice(x: usize, wall_height: f32, texture: &Texture, tex_x: f32) {
  let line_height = wall_height as i32;
  
  // Calculate start and end of line to draw on screen
  let draw_start = max(0, (-line_height / 2 + screen_height / 2));
  let draw_end = min(screen_height - 1, (line_height / 2 + screen_height / 2));
  
  // Texture mapping
  for y in draw_start..=draw_end {
    // Calculate texture Y coordinate
    let d = y * 256 - screen_height * 128 + line_height * 128;
    let tex_y = ((d * texture.height as i32) / line_height) / 256;
    
    // Sample texture and draw pixel
    let color = texture.sample(tex_x as u32, tex_y as u32);
    set_pixel(x, y as usize, color);
  }
}
```

## ⚙️ Performance Tuning

### Ray Count Adjustment

Control rendering quality and performance:

```rust
// Lower ray count = better performance, blockier graphics
let ray_count = 120; // Classic low-resolution look

// Higher ray count = better quality, slower rendering  
let ray_count = 320; // Smoother walls

// Maximum quality (1:1 pixel mapping)
let ray_count = screen_width; // Best visual quality
```

### Optimization Techniques

```rust
// Use lookup tables for trigonometric functions
struct TrigLookup {
  sin_table: [f32; 360],
  cos_table: [f32; 360],
}

// Pre-calculate common values
let delta_dist_x = direction.x.recip().abs(); // Faster than division
let delta_dist_y = direction.y.recip().abs();

// Early termination for very distant walls
if perpendicular_wall_dist > MAX_RENDER_DISTANCE {
  continue; // Skip distant walls
}
```

## 🎨 Advanced Features

### Texture Mapping
```rust
// Calculate exact texture coordinate
fn calculate_texture_x(pos: Vec2, dir: Vec2, dist: f32, side: u8) -> f32 {
  let wall_x = if side == 0 {
    pos.y + dist * dir.y
  } else {
    pos.x + dist * dir.x
  };
  
  // Convert to texture space [0, 1]
  wall_x - wall_x.floor()
}

// Apply texture with proper scaling
fn sample_wall_texture(texture: &Texture, tex_x: f32, tex_y: f32) -> Color {
  let pixel_x = (tex_x * texture.width as f32) as usize;
  let pixel_y = (tex_y * texture.height as f32) as usize;
  
  texture.get_pixel(pixel_x, pixel_y)
}
```

### Collision Detection
```rust
// Player movement with wall collision
fn move_player(player: &mut Player, world: &WorldMap, dt: f32) {
  let move_speed = 5.0 * dt;
  let new_x = player.pos.x + player.dir.x * move_speed;
  let new_y = player.pos.y + player.dir.y * move_speed;
  
  // Check X movement
  if !world.get(new_x as i32, player.pos.y as i32).is_wall() {
    player.pos.x = new_x;
  }
  
  // Check Y movement  
  if !world.get(player.pos.x as i32, new_y as i32).is_wall() {
    player.pos.y = new_y;
  }
}
```

### Multiple Wall Types
```rust
#[derive(Clone, Copy)]
enum WallType {
  Empty = 0,
  BrickWall = 1,
  StoneWall = 2,
  WoodWall = 3,
  MetalWall = 4,
}

impl WallType {
  fn get_texture(&self) -> &'static Texture {
    match self {
      WallType::BrickWall => &BRICK_TEXTURE,
      WallType::StoneWall => &STONE_TEXTURE,
      WallType::WoodWall => &WOOD_TEXTURE,
      WallType::MetalWall => &METAL_TEXTURE,
      WallType::Empty => panic!("Empty walls have no texture"),
    }
  }
}
```

## 📊 Performance Characteristics

### Ray Count Impact
| Ray Count | Visual Quality | Performance | Use Case |
|-----------|----------------|-------------|----------|
| 80-120 | Retro/Blocky | 60+ FPS | Classic feel |
| 160-240 | Balanced | 30-60 FPS | Modern smooth |
| 320+ | High Quality | 15-30 FPS | Screenshots |
| Screen Width | Maximum | Variable | Production |

### Optimization Trade-offs
- **Lower Resolution** - Better performance, chunkier pixels
- **Distance Culling** - Skip very far walls for speed
- **Simplified Textures** - Reduce memory bandwidth
- **Fixed-Point Math** - Integer operations for older hardware

## 🎯 Educational Applications

### Computer Graphics Concepts
- **3D Projection** - Converting 3D space to 2D screen
- **Spatial Traversal** - Efficient grid-based algorithms
- **Texture Mapping** - UV coordinate calculation
- **Performance Optimization** - Quality vs. speed trade-offs

### Mathematical Principles
- **Vector Operations** - Direction calculation and normalization
- **Trigonometry** - Angle-based calculations
- **Linear Interpolation** - Smooth texture coordinate mapping
- **Grid Algorithms** - Digital differential analysis

### Game Development History
- **Early 3D Graphics** - Pre-hardware acceleration techniques
- **Memory Constraints** - Optimizing for limited resources
- **Real-Time Rendering** - Achieving interactive frame rates
- **Creative Solutions** - Working within technical limitations

## 🛠️ Extending the Example

### Adding Sprites
```rust
// Simple sprite rendering
struct Sprite {
  position: Vec2,
  texture: Texture,
  scale: f32,
}

fn render_sprites(&self, sprites: &[Sprite], player: &Player) {
  for sprite in sprites {
    let relative_pos = sprite.position - player.pos;
    let distance = relative_pos.length();
    
    // Transform to screen coordinates
    let screen_x = self.transform_to_screen(relative_pos, &player.camera);
    let sprite_height = (SCREEN_HEIGHT as f32 / distance) * sprite.scale;
    
    self.draw_sprite(screen_x, sprite_height, &sprite.texture);
  }
}
```

### Multiple Floors/Ceilings
```rust
// Floor and ceiling rendering
fn render_floor_ceiling(&mut self, player: &Player) {
  for y in (SCREEN_HEIGHT / 2)..SCREEN_HEIGHT {
    // Ray direction for leftmost and rightmost ray
    let ray_dir_left = player.dir - player.plane;
    let ray_dir_right = player.dir + player.plane;
    
    // Vertical position of the row on screen
    let p = y - SCREEN_HEIGHT / 2;
    let pos_z = 0.5 * SCREEN_HEIGHT as f32;
    
    let row_distance = pos_z / p as f32;
    
    // Calculate real world coordinates
    let floor_step_x = row_distance * (ray_dir_right.x - ray_dir_left.x) / SCREEN_WIDTH as f32;
    let floor_step_y = row_distance * (ray_dir_right.y - ray_dir_left.y) / SCREEN_WIDTH as f32;
    
    let mut floor_x = player.pos.x + row_distance * ray_dir_left.x;
    let mut floor_y = player.pos.y + row_distance * ray_dir_left.y;
    
    for x in 0..SCREEN_WIDTH {
      // Sample floor texture
      let floor_color = sample_floor_texture(floor_x, floor_y);
      self.set_pixel(x, y, floor_color);
      
      // Mirror for ceiling
      let ceiling_color = sample_ceiling_texture(floor_x, floor_y);
      self.set_pixel(x, SCREEN_HEIGHT - y - 1, ceiling_color);
      
      floor_x += floor_step_x;
      floor_y += floor_step_y;
    }
  }
}
```

### Door Systems
```rust
// Animated doors
struct Door {
  position: (i32, i32),
  open_state: f32, // 0.0 = closed, 1.0 = open
  opening_speed: f32,
}

impl Door {
  fn update(&mut self, dt: f32, player_nearby: bool) {
    if player_nearby && self.open_state < 1.0 {
      self.open_state = (self.open_state + self.opening_speed * dt).min(1.0);
    } else if !player_nearby && self.open_state > 0.0 {
      self.open_state = (self.open_state - self.opening_speed * dt).max(0.0);
    }
  }
  
  fn get_wall_height(&self) -> f32 {
    1.0 - self.open_state // Doors slide upward
  }
}
```

## 📖 Learning Resources

### Raycasting Theory
- **[Lode's Computer Graphics Tutorial](https://lodev.org/cgtutor/raycasting.html)** - Comprehensive raycasting guide
- **[Wolfenstein 3D Source Code](https://github.com/id-Software/wolf3d)** - Original implementation
- **[Ray Casting Computer Graphics](https://en.wikipedia.org/wiki/Ray_casting)** - Wikipedia overview

### Graphics Programming
- **[Real-Time Rendering](http://www.realtimerendering.com/)** - Advanced rendering techniques  
- **[Computer Graphics: Principles and Practice](https://www.pearson.com/us/higher-education/program/Hughes-Computer-Graphics-Principles-and-Practice-3rd-Edition/PGM94101.html)** - Fundamental algorithms
- **[Tricks of the Game Programming Gurus](https://archive.org/details/TricksOfTheGameProgrammingGurus)** - Classic optimization techniques

## 🛠️ Troubleshooting

### Common Issues
- **Fisheye Effect** - Use perpendicular distance, not Euclidean
- **Texture Stretching** - Ensure proper UV coordinate calculation  
- **Performance Problems** - Reduce ray count or optimize inner loops
- **Wall Gaps** - Check floating-point precision in grid traversal

### Debug Techniques
```rust
// Visualize ray paths for debugging
fn debug_draw_rays(&self, player: &Player) {
  for i in 0..20 { // Draw subset of rays
    let camera_x = 2.0 * i as f32 / 20.0 - 1.0;
    let ray_dir = player.dir + player.plane * camera_x;
    
    // Draw ray line on minimap
    self.draw_line(player.pos, player.pos + ray_dir * 10.0, Color::RED);
  }
}
```

## 🎮 Game Development Applications

### Level Design
- **Grid-Based Maps** - Easy level editing and collision detection
- **Modular Content** - Tile-based world construction
- **Performance Predictability** - Consistent frame rates
- **Memory Efficiency** - Compact world representation

### Gameplay Mechanics
- **Stealth Games** - Line-of-sight calculations
- **Puzzle Games** - Ray-based interaction systems
- **Arcade Shooters** - Classic FPS movement and aiming
- **Educational Games** - Visual algorithm demonstrations

## 🤝 Contributing

Part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT
