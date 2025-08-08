# Tiles Tools

A high-performance, generic, and extensible Rust crate for developing tile-based games and applications. Built on the wTools ecosystem with strict adherence to design and codestyle principles.

## Features

- **Lightweight ECS**: Built on HECS with clean abstraction layers
- **Multiple Coordinate Systems**: Hexagonal (Axial, Offset), Square, and Pixel coordinates
- **Grid Management**: Type-safe grid operations with efficient queries
- **Pathfinding**: A* algorithm with configurable heuristics
- **Procedural Generation**: Wave Function Collapse and noise generation
- **Comprehensive Testing**: Full test coverage with integration examples

## Architecture

The library follows strict architectural principles:

- **Modular Design**: Clear separation of concerns with mod_interface patterns
- **Error Handling**: Exclusive use of error_tools for consistent error management  
- **Type Safety**: Newtype wrappers for all core types
- **Performance**: Optimized data structures with cache-friendly layouts

## Quick Start

```rust
use tiles_tools::coordinates::hexagonal::{ Coordinate, Axial, Pointy };
use tiles_tools::coordinates::{ Distance, Neighbors };

// Create axial coordinates
let coord = Coordinate::<Axial, Pointy>::new(0, 0);

// Calculate distance between coordinates
let other_coord = Coordinate::<Axial, Pointy>::new(1, 1);
let distance = coord.distance(other_coord);

// Get neighbors
let neighbors = coord.neighbors();
assert_eq!(neighbors.len(), 6);
```

## Pathfinding

```rust
use tiles_tools::pathfind::astar;
use tiles_tools::coordinates::hexagonal::{ Coordinate, Axial, Pointy };

let start = Coordinate::<Axial, Pointy>::new(0, 0);
let goal = Coordinate::<Axial, Pointy>::new(5, 5);

if let Some((path, cost)) = astar(
    &start,
    &goal,
    |coord| true, // All tiles accessible
    |_coord| 1    // Uniform cost
) {
    println!("Found path with cost: {}", cost);
}
```

## Feature Flags

- `enabled` (default): Core functionality
- `full`: All features enabled
- `integration`: Integration tests

## License

MIT