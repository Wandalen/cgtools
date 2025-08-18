//! # 🎲 Tiles Tools

#![ allow( clippy::implicit_return ) ]
#![ allow( clippy::missing_inline_in_public_items ) ]
#![ allow( clippy::default_numeric_fallback ) ]
#![ allow( clippy::missing_trait_methods ) ]
#![ allow( clippy::wildcard_imports ) ]
#![ allow( clippy::arithmetic_side_effects ) ]
#![ allow( clippy::indexing_slicing ) ]
#![ allow( clippy::panic ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::ptr_as_ptr ) ]
#![ allow( clippy::as_conversions ) ]
#![ allow( clippy::needless_maybe_sized ) ]
#![ allow( clippy::std_instead_of_core ) ]
#![ allow( clippy::blanket_clippy_restriction_lints ) ]
#![ allow( clippy::needless_return ) ]
#![ allow( clippy::transmute_ptr_to_ptr ) ]
#![ allow( clippy::elidable_lifetime_names ) ]
#![ allow( clippy::if_then_some_else_none ) ]
#![ allow( clippy::borrow_as_ptr ) ]
#![ allow( clippy::return_self_not_must_use ) ]
#![ allow( clippy::missing_docs_in_private_items ) ]
#![ allow( clippy::single_char_lifetime_names ) ]
#![ allow( clippy::module_name_repetitions ) ]
#![ allow( clippy::too_many_lines ) ]
#![ allow( clippy::missing_errors_doc ) ]
#![ allow( clippy::missing_panics_doc ) ]
#![ allow( clippy::else_if_without_else ) ]
#![ allow( clippy::similar_names ) ]
#![ allow( clippy::int_plus_one ) ]
#![ allow( clippy::must_use_candidate ) ]
#![ allow( clippy::min_ident_chars ) ]
#![ allow( clippy::uninlined_format_args ) ]
#![ allow( clippy::redundant_else ) ]
#![ allow( clippy::unreadable_literal ) ]
#![ allow( clippy::exhaustive_structs ) ]
#![ allow( clippy::exhaustive_enums ) ]
#![ allow( clippy::match_same_arms ) ]
#![ allow( clippy::float_cmp ) ]
#![ allow( clippy::manual_map ) ]
#![ allow( clippy::cast_lossless ) ]
#![ allow( clippy::cast_possible_truncation ) ]
#![ allow( clippy::cast_precision_loss ) ]
#![ allow( clippy::cast_sign_loss ) ]
#![ allow( clippy::many_single_char_names ) ]
#![ allow( clippy::unused_self ) ]
#![ allow( clippy::needless_continue ) ]
#![ allow( unused_imports ) ]
#![ allow( missing_docs ) ]
#![ allow( clippy::derivable_impls ) ]
#![ allow( clippy::missing_fields_in_debug ) ]
#![ allow( clippy::semicolon_if_nothing_returned ) ]
#![ allow( clippy::map_unwrap_or ) ]
#![ allow( clippy::std_instead_of_alloc ) ]
#![ allow( clippy::unnecessary_cast ) ]
#![ allow( clippy::unnecessary_semicolon ) ]
#![ allow( clippy::only_used_in_recursion ) ]
#![ allow( clippy::no_effect_underscore_binding ) ]
#![ allow( clippy::trivially_copy_pass_by_ref ) ]
#![ allow( clippy::explicit_iter_loop ) ]
#![ allow( clippy::clone_on_copy ) ]
#![ allow( dead_code ) ]
#![ allow( clippy::redundant_closure_for_method_calls ) ]
#![ allow( clippy::from_over_into ) ]
#![ allow( clippy::iter_without_into_iter ) ]
#![ allow( clippy::cast_possible_wrap ) ]
#![ allow( clippy::mem_replace_option_with_some ) ]
#![ allow( clippy::needless_range_loop ) ]
#![ allow( clippy::non_canonical_clone_impl ) ]
#![ allow( clippy::needless_pass_by_value ) ]
#![ allow( clippy::default_trait_access ) ]
#![ allow( clippy::cast_abs_to_unsigned ) ]
#![ allow( clippy::useless_conversion ) ]
#![ allow( clippy::needless_raw_string_hashes ) ]
#![ allow( clippy::format_push_string ) ]
#![ allow( clippy::doc_markdown ) ]
#![ allow( clippy::type_complexity ) ]
#![ allow( clippy::if_not_else ) ]

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
//! # #[ cfg( feature = "enabled" ) ]
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

#[ cfg( feature = "enabled" ) ]
pub mod collection;

#[ cfg( feature = "enabled" ) ]
pub mod geometry;

#[ cfg( feature = "enabled" ) ]
pub mod pathfind;

#[ cfg( feature = "enabled" ) ]
pub mod layout;

#[ cfg( feature = "enabled" ) ]
pub mod ecs;

#[ cfg( feature = "enabled" ) ]
pub mod flowfield;

#[ cfg( feature = "enabled" ) ]
pub mod field_of_view;

#[ cfg( feature = "enabled" ) ]
pub mod spatial;

#[ cfg( feature = "enabled" ) ]
pub mod animation;

#[ cfg( feature = "enabled" ) ]
pub mod events;

#[ cfg( feature = "serialization" ) ]
pub mod serialization;

#[ cfg( feature = "enabled" ) ]
pub mod debug;

#[ cfg( feature = "enabled" ) ]
pub mod game_systems;
