# Changelog

All notable changes to the `tiles_tools` crate will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

## 0.1.0 - 2024-01-XX

### Added
- **Complete coordinate system support**
  - Hexagonal coordinates (Axial, Offset) with pointy and flat orientations
  - Square coordinates with 4-connected and 8-connected neighbor patterns
  - Triangular coordinates with 12-connected neighbor patterns
  - Isometric coordinates with diamond projection
  - Pixel coordinates for screen-space transformations

- **Universal coordinate conversions**
  - Exact conversions between compatible systems (Square ↔ Isometric)
  - Approximate conversions for all coordinate system pairs
  - Batch conversion utilities for performance
  - Roundtrip conversion testing utilities

- **Comprehensive distance calculations**
  - Manhattan distance for square grids
  - Chebyshev distance for 8-connected square grids
  - Hexagonal distance for axial coordinate systems
  - Triangular distance calculations
  - Isometric distance calculations

- **Universal neighbor finding**
  - 6-neighbor patterns for hexagonal grids
  - 4-neighbor and 8-neighbor patterns for square grids
  - 12-neighbor patterns for triangular grids
  - 4-neighbor patterns for isometric grids

- **Pathfinding algorithms**
  - A* pathfinding working with all coordinate systems
  - Configurable heuristics and cost functions
  - Obstacle avoidance support
  - Variable terrain cost support

- **Entity-Component-System (ECS) integration**
  - Position, Movable, Health, Stats, Team components
  - AI, Animation, PlayerControlled components  
  - Movement, Combat, AI, Animation systems
  - Equipment and abilities system for RPG games
  - Initiative-based turn management

- **Field of view calculations**
  - Multiple FOV algorithms (shadowcasting, raycasting)
  - Light source management and calculations
  - Visibility state tracking
  - Line of sight calculations

- **Flow field pathfinding**
  - Multi-unit pathfinding optimization
  - Dynamic flow field updates
  - Integration field calculations
  - Efficient path following for crowds

- **Grid collection utilities**
  - Type-safe 2D grid data structures
  - Efficient coordinate-based indexing
  - Iterator support for grid traversal
  - Memory-efficient sparse grid support

- **Comprehensive examples**
  - Conway's Game of Life implementation
  - Stealth game with line-of-sight mechanics
  - Tactical RPG with turn-based combat
  - Complete game loop demonstrations

- **Extensive testing**
  - Unit tests for all coordinate systems
  - Integration tests for pathfinding
  - Property-based testing for conversions
  - Comprehensive test matrix coverage

- **Performance optimizations**
  - Zero-cost abstractions for coordinate systems
  - Efficient neighbor calculations
  - Cache-friendly data structures
  - SIMD-friendly operations where applicable

### Features
- `enabled` (default): Core functionality with all coordinate systems
- `full`: All features enabled for maximum functionality
- `serialization`: Serde support for coordinates and components
- `ecs-systems`: Enhanced ECS systems and components
- `coordinates`: Base coordinate system support
- `hexagonal`: Hexagonal coordinate system
- `square`: Square coordinate system  
- `triangular`: Triangular coordinate system
- `isometric`: Isometric coordinate system
- `conversions`: Coordinate conversion utilities
- `pathfinding-algorithms`: A* and other pathfinding algorithms
- `grid-collections`: Grid data structure utilities
- `field-of-view`: Field of view calculation systems
- `flow-fields`: Flow field pathfinding systems

### Documentation
- Complete API documentation with examples
- Comprehensive README with usage examples
- Architecture and design documentation
- Performance benchmarking suite
- Example gallery with complete game implementations

### Performance
- Benchmarks for all coordinate operations
- Pathfinding performance across different grid types
- Memory usage optimization
- Cross-platform performance validation