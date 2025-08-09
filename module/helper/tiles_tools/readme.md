# 🎲 Tiles Tools

[![Crates.io](https://img.shields.io/crates/v/tiles_tools.svg)](https://crates.io/crates/tiles_tools)
[![Documentation](https://docs.rs/tiles_tools/badge.svg)](https://docs.rs/tiles_tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)

**High-Performance Tile-Based Game Development Toolkit**

A comprehensive, generic, and extensible Rust crate for developing sophisticated tile-based games and applications. This crate provides a complete toolkit for working with multiple coordinate systems, pathfinding, ECS integration, and advanced grid-based algorithms.

## ✨ Core Features

- **🗺️ Universal Coordinate Systems**: Hexagonal, Square, Triangular, Isometric, and Pixel coordinates
- **🔄 Seamless Conversions**: Exact and approximate conversions between coordinate systems  
- **🧭 Advanced Pathfinding**: A* algorithm optimized for all coordinate systems
- **⚡ ECS Integration**: Complete Entity-Component-System with game-specific components
- **👁️ Field of View**: Multiple FOV algorithms including shadowcasting and raycasting
- **🌊 Flow Fields**: Efficient multi-unit pathfinding and crowd simulation
- **🎯 Grid Collections**: Type-safe, high-performance grid data structures
- **🚀 Zero-Cost Abstractions**: Performance-focused design with compile-time optimizations

## 🏗️ Architecture

The library follows strict architectural principles:

- **🔧 Modular Design**: Clear separation of concerns with mod_interface patterns
- **🛡️ Error Handling**: Exclusive use of error_tools for consistent error management  
- **🔐 Type Safety**: Newtype wrappers for all core types
- **🚀 Performance**: Optimized data structures with cache-friendly layouts

## 🚀 Quick Start

Add this to your `Cargo.toml`:

```toml
[dependencies]
tiles_tools = "0.1.0"
```

### 🔷 Hexagonal Coordinates
```rust
use tiles_tools::coordinates::hexagonal::{ Coordinate, Axial, Pointy };
use tiles_tools::coordinates::{ Distance, Neighbors };

let coord = Coordinate::<Axial, Pointy>::new(2, -1);
let other_coord = Coordinate::<Axial, Pointy>::new(5, 1);
let distance = coord.distance(other_coord); // Hexagonal distance
let neighbors = coord.neighbors(); // 6 surrounding hexes
assert_eq!(neighbors.len(), 6);
```

### 🔄 Coordinate Conversions

```rust
use tiles_tools::coordinates::conversion::{ Convert, ApproximateConvert };
use tiles_tools::coordinates::{ 
    square::{ Coordinate as Square, FourConnected }, 
    isometric::{ Coordinate as Iso, Diamond }
};

let square = Square::<FourConnected>::new(3, 4);
let iso: Iso<Diamond> = square.convert(); // Exact conversion
let back: Square<FourConnected> = iso.convert(); // Perfect roundtrip
assert_eq!(square, back);
```

## 🧭 Universal Pathfinding

```rust
use tiles_tools::pathfind::astar;
use tiles_tools::coordinates::square::{ Coordinate, FourConnected };

let start = Coordinate::<FourConnected>::new(0, 0);
let goal = Coordinate::<FourConnected>::new(10, 10);

if let Some((path, cost)) = astar(&start, &goal, |_| true, |_| 1) {
    println!("Found path with cost: {}", cost);
}
```

## 🎮 ECS Game Development

```rust
use tiles_tools::ecs::{ World, Position, Health, Movable };
use tiles_tools::coordinates::square::{ Coordinate, FourConnected };

let mut world = World::new();
let player = world.spawn((
    Position::new(Coordinate::<FourConnected>::new(0, 0)),
    Health::new(100),
    Movable::new(3),
));
```

## 🎲 Coordinate Systems

All coordinate systems implement the [`Distance`] and [`Neighbors`] traits, providing a uniform interface:

- **Hexagonal**: Perfect for strategy games and organic movement patterns
- **Square**: Classic grid games with 4 or 8-connected movement  
- **Triangular**: Unique tessellation with rich neighbor relationships
- **Isometric**: Pseudo-3D visualization for RPGs and city builders
- **Pixel**: Screen-space coordinates for rendering and input handling

## 📦 Feature Flags

- **`enabled`** (default): Core functionality with all coordinate systems
- **`full`**: All features for maximum functionality  
- **`ecs-systems`**: Enhanced ECS components and systems
- **`serialization`**: Serde support for save/load functionality
- **`pathfinding-algorithms`**: A* and other pathfinding algorithms
- **`field-of-view`**: Line of sight and visibility calculations
- **`flow-fields`**: Multi-unit pathfinding and crowd behavior

## 🛠️ Examples

The crate includes comprehensive examples:

- **Conway's Game of Life**: Cellular automaton implementation
- **Stealth Game**: Line-of-sight mechanics and field of view
- **Tactical RPG**: Turn-based combat with pathfinding

Run examples with:
```bash
cargo run --example game_of_life --features enabled
cargo run --example stealth_game --features enabled,ecs-systems
cargo run --example tactical_rpg --features enabled,ecs-systems,pathfinding-algorithms
```

## 🎮 Use Cases

- **Strategy Games**: Turn-based and real-time strategy games
- **RPG Systems**: Grid-based movement and tactical combat
- **Puzzle Games**: Match-3, Tetris-like, and spatial puzzles
- **Board Game Simulations**: Digital versions of classic board games
- **Map Editors**: Tools for creating tile-based worlds
- **Procedural Generation**: Algorithmic world and dungeon generation

## 📚 Documentation

- [API Documentation](https://docs.rs/tiles_tools)
- [Repository](https://github.com/Wandalen/cgtools/tree/master/module/helper/tiles_tools)

## 📄 License

This project is licensed under the MIT License - see the [license](license) file for details.