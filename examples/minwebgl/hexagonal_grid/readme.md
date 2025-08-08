# 🏺 Hexagonal Grid WebGL Example

> **Interactive hexagonal grid system with A* pathfinding and mouse interaction**

A comprehensive demonstration of hexagonal grid implementation using WebGL and Rust. Features real-time pathfinding, mouse interaction, and efficient rendering of hex-based maps. Inspired by [Red Blob Games](https://www.redblobgames.com/grids/hexagons/) hex grid theory.

![Hexagonal Grid Screenshot](./showcase.png)

## ✨ Features

### 🎮 **Interactive Elements**
- **Mouse Interaction** - Click to select start/end points for pathfinding
- **Real-Time Pathfinding** - A* algorithm with instant visual feedback
- **Grid Highlighting** - Visual indicators for selected tiles and paths
- **Responsive Controls** - Smooth interaction with grid coordinates

### 🔧 **Technical Capabilities**
- **Multiple Grid Layouts** - Pointy-topped and flat-topped hex orientations
- **Coordinate Systems** - Axial, cube, and offset coordinate implementations
- **WebGL Rendering** - GPU-accelerated hex tile rendering with custom shaders
- **Dynamic Mesh Generation** - Runtime hex mesh creation and optimization
- **Memory Efficient** - Instanced rendering for thousands of hex tiles

### 🧮 **Mathematical Foundation**
- **Coordinate Conversion** - Seamless conversion between coordinate systems
- **Distance Calculations** - Accurate hex distance metrics
- **Neighbor Finding** - Efficient adjacent hex tile lookup
- **Path Optimization** - Shortest path calculation with obstacle avoidance

## 🚀 Quick Start

### Prerequisites
- Rust with `wasm32-unknown-unknown` target
- Trunk for development server

### Run the Example
```bash
# Navigate to the hexagonal grid example
cd examples/minwebgl/hexagonal_grid

# Install trunk if not already installed
cargo install trunk

# Serve with hot reload
trunk serve --release
```

Open http://localhost:8080 in your browser to interact with the hex grid.

## 🎮 How to Use

### Basic Interaction
1. **Left Click** - Set start point (green hex)
2. **Right Click** - Set destination point (red hex)
3. **Path Display** - Optimal path shown in blue hexes
4. **Real-Time Updates** - Path recalculates instantly when points change

### Grid Navigation
- The grid uses axial coordinates for internal calculations
- Hover over hexes to see coordinate information
- Path length and calculation time displayed in real-time

## 🔧 Technical Implementation

### Coordinate Systems
```rust
// Axial coordinates (q, r)
let hex_coord = HexCoord::new(2, -1);

// Convert to pixel coordinates for rendering
let pixel_pos = hex_coord.to_pixel(hex_size);

// Find neighbors for pathfinding
let neighbors = hex_coord.neighbors();
```

### Pathfinding Algorithm
- **A* Implementation** - Optimal pathfinding with heuristic
- **Hex Distance Heuristic** - Manhattan distance in hex space
- **Obstacle Support** - Easy to extend with blocked tiles
- **Performance Optimized** - Efficient priority queue and visited set

### Rendering Pipeline
1. **Vertex Generation** - Create hex vertices in world space
2. **Instance Data** - Position, color, and state per hex
3. **GPU Rendering** - WebGL instanced drawing for performance
4. **Fragment Shading** - Custom colors for different hex states

## 📚 Key Files

| File | Purpose |
|------|---------|
| `src/lib.rs` | Main application logic and WebGL setup |
| `src/hex_grid.rs` | Hex coordinate systems and grid management |
| `src/pathfinding.rs` | A* pathfinding implementation |
| `shaders/main.vert` | Vertex shader for hex rendering |
| `shaders/main.frag` | Fragment shader for hex coloring |
| `index.html` | HTML entry point with canvas |

## 🎯 Use Cases

### Game Development
- **Tactical RPGs** - Turn-based combat on hex grids
- **Strategy Games** - Territory control and unit movement
- **Board Games** - Digital implementations of hex-based games
- **Roguelike Dungeons** - Procedural hex-based level generation

### Data Visualization
- **Geographic Data** - Hex bin maps for spatial analysis
- **Network Topology** - Visualizing connected systems
- **Cellular Automata** - Hex-based simulation systems
- **Scientific Modeling** - Molecular or crystal structure visualization

### Educational Applications
- **Algorithm Teaching** - Visual pathfinding and graph algorithms
- **Coordinate Systems** - Interactive math education
- **Computer Graphics** - WebGL and rendering pipeline demos

## 🔗 Learning Resources

- **[Red Blob Games - Hex Grids](https://www.redblobgames.com/grids/hexagons/)** - Comprehensive hex grid theory
- **[A* Pathfinding](https://www.redblobgames.com/pathfinding/a-star/)** - Pathfinding algorithm explanation
- **[WebGL Fundamentals](https://webglfundamentals.org/)** - WebGL rendering basics
- **[Tiles Tools Documentation](../../../module/helper/tiles_tools/readme.md)** - Core hex grid library

## 🛠️ Extending the Example

### Adding New Features
```rust
// Add hex obstacles
grid.set_blocked(HexCoord::new(3, 2), true);

// Custom pathfinding costs
let path = pathfind_with_cost(start, goal, |hex| {
  if is_difficult_terrain(hex) { 2.0 } else { 1.0 }
});

// Multiple path visualization
for (i, path) in paths.iter().enumerate() {
  render_path(path, COLORS[i % COLORS.len()]);
}
```

### Performance Optimization
- **Spatial Partitioning** - Divide large grids into chunks
- **Level of Detail** - Render distant hexes with less detail
- **Culling** - Skip off-screen hex rendering
- **Batch Updates** - Group pathfinding requests

## 🤝 Contributing

Part of the CGTools workspace. Feel free to submit issues and pull requests on GitHub.

## 📄 License

MIT
