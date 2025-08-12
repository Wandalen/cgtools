# 🎲 Tiles Tools

[![Crates.io](https://img.shields.io/crates/v/tiles_tools.svg)](https://crates.io/crates/tiles_tools)
[![Documentation](https://docs.rs/tiles_tools/badge.svg)](https://docs.rs/tiles_tools)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://github.com/Wandalen/cgtools/blob/master/module/helper/tiles_tools/license)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-blue.svg)](https://www.rust-lang.org)

**A high-performance, generic, and extensible Rust crate for developing sophisticated tile-based games and applications.**

This crate provides a complete toolkit for working with multiple coordinate systems, pathfinding, ECS integration, advanced AI systems, and performance-optimized game mechanics.

## ✨ Core Features

*   **Universal Coordinate Systems**: First-class support for Hexagonal, Square, Triangular, and Isometric grids.
*   **Advanced Pathfinding**: A\* algorithm optimized for all coordinate systems with support for obstacles and variable terrain costs.
*   **Complete Game Framework**: A full Entity-Component-System (ECS) powered by `hecs` with specialized components and systems for grid-based games.
*   **Performance-Optimized Systems**: Includes an animation system, behavior trees for AI, a type-safe event system, and spatial partitioning to ensure high performance.
*   **Development Tools**: Features debug visualization with ASCII and SVG output, performance profiling, and comprehensive serialization for save/load functionality.

## 🚀 Quick Start

Add `tiles_tools` to your `Cargo.toml`:

```toml
[dependencies]
tiles_tools = "0.1.0"
```

Create a simple game world with pathfinding:

```rust
use tiles_tools::
{
  coordinates::square::{ Coordinate, FourConnected },
  ecs::{ World, Position, Health },
  pathfind::astar,
  game_systems::{ TurnBasedGame, ResourceManager },
  debug::GridRenderer,
};

fn main()
{
  // Create game world and systems
  let mut world = World::new();
  let mut turn_game = TurnBasedGame::new();
  let mut resource_manager = ResourceManager::new();

  // Spawn player entity
  let player = world.spawn( (
    Position::new( Coordinate::< FourConnected >::new( 1, 1 ) ),
    Health::new( 100 ),
  ) );

  // Add to turn-based system
  turn_game.add_participant( player.id() as u32, 100 );
  resource_manager.add_entity( player.id() as u32, 100.0, 30.0 );

  // Pathfinding with obstacle avoidance
  let start = Coordinate::< FourConnected >::new( 1, 1 );
  let goal = Coordinate::< FourConnected >::new( 10, 8 );

  if let Some( ( path, cost ) ) = astar( &start, &goal, | _ | true, | _ | 1 )
  {
    println!( "Found path with {} steps, cost: {}", path.len(), cost );
  }

  // Debug visualization
  let mut debug_renderer = GridRenderer::new()
  .with_size( 12, 10 )
  .with_style( tiles_tools::debug::GridStyle::Square4 );

  debug_renderer.add_colored_marker
  (
    ( 1, 1 ),
    "P",
    "Player",
    tiles_tools::debug::DebugColor::Green,
    20
  );
  println!( "\n{}", debug_renderer.render_ascii() );
}
```

## 📦 Examples

The crate includes a wide range of examples to demonstrate its capabilities.

| Example | Description | Command |
|---|---|---|
| **beginner\_tutorial** | A step-by-step guide to the core concepts. | `cargo run --example beginner_tutorial` |
| **tactical\_rpg** | A complete hexagonal grid tactical combat game. | `cargo run --example tactical_rpg` |
| **stealth\_game** | Demonstrates field-of-view and lighting mechanics. | `cargo run --example stealth_game` |
| **behavior\_tree\_demo** | Showcases the advanced AI decision-making system. | `cargo run --example behavior_tree_demo` |
| **serialization\_demo** | Implements save/load functionality. | `cargo run --example serialization_demo --features serialization` |

To run an example, use the command `cargo run --example <example_name>`. Some examples may require specifying features.

---

**Built with ❤️ for the Rust game development community**

*Tiles Tools - Making tile-based game development simple, fast, and fun.*