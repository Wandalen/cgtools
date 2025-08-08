# 🎲 Tiles Tools

**High-Performance Tile-Based Game Development**

A comprehensive, generic, and extensible Rust crate for developing sophisticated tile-based games and applications. Built on the wTools ecosystem with strict adherence to design and codestyle principles, providing a solid foundation for your next game project.

## ✨ Features

- **⚡ Lightweight ECS**: Built on HECS with clean abstraction layers
- **🗺️ Multiple Coordinate Systems**: Hexagonal (Axial, Offset), Square (4/8-connected), Triangular (12-connected), Isometric (Diamond), and Pixel coordinates
- **🎯 Grid Management**: Type-safe grid operations with efficient queries
- **🧭 Pathfinding**: A* algorithm with configurable heuristics
- **🎨 Procedural Generation**: Wave Function Collapse and noise generation
- **✅ Comprehensive Testing**: Full test coverage with integration examples

## 🏗️ Architecture

The library follows strict architectural principles:

- **🔧 Modular Design**: Clear separation of concerns with mod_interface patterns
- **🛡️ Error Handling**: Exclusive use of error_tools for consistent error management  
- **🔐 Type Safety**: Newtype wrappers for all core types
- **🚀 Performance**: Optimized data structures with cache-friendly layouts

## 🚀 Quick Start

### 🔷 Hexagonal Coordinates
```rust
use tiles_tools::coordinates::hexagonal::{ Coordinate, Axial, Pointy };
use tiles_tools::coordinates::{ Distance, Neighbors };

// Create axial coordinates
let coord = Coordinate::<Axial, Pointy>::new(0, 0);

// Calculate distance between coordinates
let other_coord = Coordinate::<Axial, Pointy>::new(1, 1);
let distance = coord.distance(&other_coord);

// Get neighbors
let neighbors = coord.neighbors();
assert_eq!(neighbors.len(), 6);
```

### ⬛ Square Coordinates
```rust
use tiles_tools::coordinates::square::{ Coordinate, FourConnected, EightConnected };
use tiles_tools::coordinates::{ Distance, Neighbors };

// 4-connected square grid (orthogonal movement only)
let coord4 = Coordinate::<FourConnected>::new(2, 3);
let neighbors4 = coord4.neighbors();
assert_eq!(neighbors4.len(), 4); // Up, Down, Left, Right

// Manhattan distance
let other4 = Coordinate::<FourConnected>::new(5, 7);
let manhattan_dist = coord4.distance(&other4); // |5-2| + |7-3| = 7

// 8-connected square grid (includes diagonals)
let coord8 = Coordinate::<EightConnected>::new(2, 3);
let neighbors8 = coord8.neighbors();
assert_eq!(neighbors8.len(), 8); // All surrounding cells

// Chebyshev distance  
let other8 = Coordinate::<EightConnected>::new(5, 7);
let chebyshev_dist = coord8.distance(&other8); // max(|5-2|, |7-3|) = 4
```

### 🔺 Triangular Coordinates
```rust
use tiles_tools::coordinates::triangular::{ TriangularCoord, TwelveConnected };
use tiles_tools::coordinates::{ Distance, Neighbors };

// Triangular grid with 12-neighbor connectivity
let coord = TriangularCoord::new(2, 3);
let neighbors = coord.neighbors();
assert_eq!(neighbors.len(), 12); // 3 edge-adjacent + 9 vertex-adjacent

// Check triangle orientation
assert!(coord.is_down_pointing()); // (2+3)%2 == 1 -> down triangle ▽

// Calculate triangular distance
let other = TriangularCoord::new(5, 7);
let distance = coord.distance(&other); // max(|5-2|, |7-3|) = 4
```

### 💎 Isometric Coordinates
```rust
use tiles_tools::coordinates::isometric::{ IsometricCoord, Diamond };
use tiles_tools::coordinates::{ Distance, Neighbors, pixel::Pixel };

// Isometric grid for pseudo-3D visualization
let coord = IsometricCoord::new(3, 2);
let neighbors = coord.neighbors();
assert_eq!(neighbors.len(), 4); // Orthogonal neighbors

// Transform to screen coordinates for rendering
let screen_pos: Pixel = coord.to_screen(32.0); // 32 pixels per tile

// Convert screen coordinates back to world coordinates  
let world_coord = IsometricCoord::from_screen(screen_pos, 32.0);
assert_eq!(world_coord, coord);

// Get diamond tile corners for rendering
let corners = coord.tile_corners(32.0);
assert_eq!(corners.len(), 4); // Top, Right, Bottom, Left
```

### 🔄 Universal Coordinate Conversions
```rust
use tiles_tools::coordinates::conversion::{Convert, ApproximateConvert, BatchConvertExact};
use tiles_tools::coordinates::{
    square::{Coordinate as SquareCoord, FourConnected},
    isometric::{Coordinate as IsoCoord, Diamond},
    hexagonal::{Coordinate as HexCoord, Axial, Pointy},
    triangular::TriangularCoord,
};

// Exact conversion: Square ↔ Isometric (no information loss)
let square = SquareCoord::<FourConnected>::new(3, 2);
let iso: IsoCoord<Diamond> = square.convert();
let back: SquareCoord<FourConnected> = iso.convert();
assert_eq!(square, back); // Perfect roundtrip

// Approximate conversion: Hexagonal ↔ Square (best effort)
let hex = HexCoord::<Axial, Pointy>::new(2, -1);
let square_approx: SquareCoord<FourConnected> = hex.approximate_convert();

// Batch conversion for performance
let squares = vec![
    SquareCoord::<FourConnected>::new(0, 0),
    SquareCoord::<FourConnected>::new(1, 1),
    SquareCoord::<FourConnected>::new(2, 2),
];
let isos: Vec<IsoCoord<Diamond>> = squares.convert_batch_exact();
```

## 🧭 Universal Pathfinding

The A* algorithm works with any coordinate system:

```rust
use tiles_tools::pathfind::astar;
use tiles_tools::coordinates::{
    hexagonal::{ Coordinate as HexCoord, Axial, Pointy },
    square::{ Coordinate as SquareCoord, FourConnected },
    triangular::TriangularCoord,
    isometric::IsometricCoord,
};

// Pathfinding on hexagonal grid
let hex_start = HexCoord::<Axial, Pointy>::new(0, 0);
let hex_goal = HexCoord::<Axial, Pointy>::new(5, 5);

if let Some((path, cost)) = astar(&hex_start, &hex_goal, |_| true, |_| 1) {
    println!("Hex path cost: {}", cost);
}

// Pathfinding on triangular grid
let tri_start = TriangularCoord::new(1, 1);
let tri_goal = TriangularCoord::new(4, 7);

if let Some((path, cost)) = astar(&tri_start, &tri_goal, |_| true, |_| 1) {
    println!("Triangular path cost: {}", cost);
}

// Pathfinding on isometric grid
let iso_start = IsometricCoord::new(0, 0);
let iso_goal = IsometricCoord::new(3, 4);

if let Some((path, cost)) = astar(&iso_start, &iso_goal, |_| true, |_| 1) {
    println!("Isometric path cost: {}", cost);
}

// Pathfinding on square grid
let square_start = SquareCoord::<FourConnected>::new(0, 0);
let square_goal = SquareCoord::<FourConnected>::new(5, 5);

if let Some((path, cost)) = astar(&square_start, &square_goal, |_| true, |_| 1) {
    println!("Square path cost: {}", cost);
}
```

## ⚙️ Feature Flags

- `enabled` (default): Core functionality
- `full`: All features enabled
- `integration`: Integration tests

## 🚀 Getting Started

Add this to your `Cargo.toml`:

```toml
[dependencies]
tiles_tools = "0.1.0"
```

## 📖 Documentation

For detailed API documentation and examples, run:

```bash
cargo doc --open
```

## 🎮 Use Cases

- **Strategy Games**: Turn-based and real-time strategy games
- **RPG Systems**: Grid-based movement and tactical combat
- **Puzzle Games**: Match-3, Tetris-like, and spatial puzzles
- **Board Game Simulations**: Digital versions of classic board games
- **Map Editors**: Tools for creating tile-based worlds
- **Procedural Generation**: Algorithmic world and dungeon generation

## 📄 License

MIT