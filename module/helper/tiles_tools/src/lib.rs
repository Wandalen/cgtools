//! # 🎲 Tiles Tools
//! 
//! **High-Performance Tile-Based Game Development Toolkit**
//!
//! A comprehensive, generic, and extensible Rust crate for developing sophisticated 
//! tile-based games and applications. This crate provides a complete toolkit for 
//! working with multiple coordinate systems, pathfinding, ECS integration, and 
//! advanced grid-based algorithms.
//!
//! ## ✨ Core Features
//!
//! - **🗺️ Universal Coordinate Systems**: Hexagonal, Square, Triangular, Isometric, and Pixel coordinates
//! - **🔄 Seamless Conversions**: Exact and approximate conversions between coordinate systems  
//! - **🧭 Advanced Pathfinding**: A* algorithm optimized for all coordinate systems
//! - **⚡ ECS Integration**: Complete Entity-Component-System with game-specific components
//! - **👁️ Field of View**: Multiple FOV algorithms including shadowcasting and raycasting
//! - **🌊 Flow Fields**: Efficient multi-unit pathfinding and crowd simulation
//! - **🎯 Grid Collections**: Type-safe, high-performance grid data structures
//! - **🚀 Zero-Cost Abstractions**: Performance-focused design with compile-time optimizations
//!
//! ## 🚀 Quick Start
//!
//! ### Hexagonal Grids
//! ```rust
//! use tiles_tools::coordinates::hexagonal::{ Coordinate, Axial, Pointy };
//! use tiles_tools::coordinates::{ Distance, Neighbors };
//!
//! let coord = Coordinate::<Axial, Pointy>::new(2, -1);
//! let other_coord = Coordinate::<Axial, Pointy>::new(5, 1);
//! let distance = coord.distance(other_coord); // Hexagonal distance
//! let neighbors = coord.neighbors(); // 6 surrounding hexes
//! assert_eq!(neighbors.len(), 6);
//! ```
//!
//! ### Universal Pathfinding
//! ```rust
//! use tiles_tools::pathfind::astar;
//! use tiles_tools::coordinates::square::{ Coordinate, FourConnected };
//!
//! let start = Coordinate::<FourConnected>::new(0, 0);
//! let goal = Coordinate::<FourConnected>::new(10, 10);
//!
//! if let Some((path, cost)) = astar(&start, &goal, |_| true, |_| 1) {
//!     println!("Found path with cost: {}", cost);
//! }
//! ```
//!
//! ### ECS Game Development
//! ```rust
//! # #[cfg(feature = "enabled")]
//! # {
//! use tiles_tools::ecs::{ World, Position, Health, Movable };
//! use tiles_tools::coordinates::square::{ Coordinate, FourConnected };
//!
//! let mut world = World::new();
//! let player = world.spawn((
//!     Position::new(Coordinate::<FourConnected>::new(0, 0)),
//!     Health::new(100),
//!     Movable::new(3),
//! ));
//! # }
//! ```
//!
//! ## 🎮 Coordinate Systems
//!
//! All coordinate systems implement the [`Distance`](coordinates::Distance) and 
//! [`Neighbors`](coordinates::Neighbors) traits, providing a uniform interface:
//!
//! - **Hexagonal**: Perfect for strategy games and organic movement patterns
//! - **Square**: Classic grid games with 4 or 8-connected movement  
//! - **Triangular**: Unique tessellation with rich neighbor relationships
//! - **Isometric**: Pseudo-3D visualization for RPGs and city builders
//! - **Pixel**: Screen-space coordinates for rendering and input handling
//!
//! ## 🔄 Coordinate Conversions
//!
//! Convert between coordinate systems with exact or approximate transformations:
//!
//! ```rust
//! use tiles_tools::coordinates::conversion::{ Convert, ApproximateConvert };
//! use tiles_tools::coordinates::{ 
//!     square::{ Coordinate as Square, FourConnected }, 
//!     isometric::{ Coordinate as Iso, Diamond }
//! };
//!
//! let square = Square::<FourConnected>::new(3, 4);
//! let iso: Iso<Diamond> = square.convert(); // Exact conversion
//! let back: Square<FourConnected> = iso.convert(); // Perfect roundtrip
//! assert_eq!(square, back);
//! ```
//!
//! ## 📦 Feature Flags
//!
//! - **`enabled`** (default): Core functionality with all coordinate systems
//! - **`full`**: All features for maximum functionality  
//! - **`ecs-systems`**: Enhanced ECS components and systems
//! - **`serialization`**: Serde support for save/load functionality
//! - **`pathfinding-algorithms`**: A* and other pathfinding algorithms
//! - **`field-of-view`**: Line of sight and visibility calculations
//! - **`flow-fields`**: Multi-unit pathfinding and crowd behavior
//!
//! ## 🏗️ Architecture
//!
//! This crate is built on solid architectural principles:
//!
//! - **Generic Design**: All algorithms work across coordinate systems
//! - **Zero-Cost Abstractions**: Compile-time polymorphism for performance
//! - **Modular Structure**: Use only the components you need
//! - **Type Safety**: Prevent coordinate system mixing errors at compile time
//! - **Memory Efficiency**: Cache-friendly data structures and algorithms

#![ cfg_attr( not( feature = "enabled" ), allow( unused ) ) ]

pub mod coordinates;

#[cfg(feature = "enabled")]
pub mod collection;

#[cfg(feature = "enabled")]
pub mod geometry;

#[cfg(feature = "enabled")]
pub mod pathfind;

#[cfg(feature = "enabled")]
pub mod layout;

#[cfg(feature = "enabled")]
pub mod ecs;

#[cfg(feature = "enabled")]
pub mod flowfield;

#[cfg(feature = "enabled")]
pub mod field_of_view;

#[cfg(feature = "enabled")]
pub mod spatial;

#[cfg(feature = "enabled")]
pub mod behavior_tree;

#[cfg(feature = "enabled")]
pub mod animation;
