//! Entity-Component-System integration for tile-based games and applications.
//!
//! This module provides a comprehensive ECS implementation built on top of HECS,
//! with specialized components and systems for working with tile grids and
//! coordinate systems.
//!
//! # Architecture
//!
//! The ECS architecture follows these principles:
//! - **Entities**: Unique identifiers for game objects
//! - **Components**: Pure data structures describing entity properties
//! - **Systems**: Logic that operates on entities with specific component combinations
//! - **World**: Container managing entities, components, and system execution
//!
//! # Grid-Aware Design
//!
//! This ECS implementation is specifically designed for tile-based games:
//! - Position components support all coordinate systems (Hexagonal, Square, Triangular, Isometric)
//! - Movement systems understand grid constraints and pathfinding
//! - Spatial systems provide efficient neighbor queries and collision detection
//! - Rendering systems handle coordinate-to-screen transformations
//!
//! # Examples
//!
//! ```rust
//! use tiles_tools::ecs::{ World, Position, Movable, Health };
//! use tiles_tools::coordinates::square::{ Coordinate as SquareCoord, FourConnected };
//! 
//! // Create a world and spawn entities
//! let mut world = World::new();
//! 
//! let player = world.spawn((
//!     Position::new( SquareCoord::< FourConnected >::new( 0, 0 ) ),
//!     Movable::new( 2 ), // 2 tiles per turn
//!     Health::new( 100 ),
//! ));
//! 
//! let enemy = world.spawn((
//!     Position::new( SquareCoord::< FourConnected >::new( 5, 3 ) ),
//!     Health::new( 50 ),
//! ));
//! 
//! // Query entities with specific components
//! for ( entity, ( pos, health ) ) in world.query::< ( &Position< SquareCoord< FourConnected > >, &Health ) >().iter()
//! {
//!     println!( "Entity {:?} at {:?} has {} health", entity, pos.coord, health.current );
//! }
//! ```

pub mod components;
pub mod systems;
pub mod world;

pub use components::*;
pub use systems::*;
pub use world::*;