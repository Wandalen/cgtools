# Tiles Tools Engine Specification

**Name:** Tiles Tools Engine  
**Version:** 1.1 (Rulebook Compliant)  
**Date:** 2025-08-08

## Table of Contents

**Part I: Public Contract (Mandatory Requirements)**
1. [Project Goal](#1-project-goal)
2. [Problem Solved](#2-problem-solved)
3. [Ubiquitous Language](#3-ubiquitous-language)
4. [Deliverables](#4-deliverables)
5. [Functional Requirements](#5-functional-requirements)
6. [Non-Functional Requirements](#6-non-functional-requirements)
7. [Dependency Architecture](#7-dependency-architecture)

**Part II: Internal Design (Rulebook Compliant)**
8. [Module Structure](#8-module-structure)
9. [Error Handling Strategy](#9-error-handling-strategy)
10. [Code Generation Strategy](#10-code-generation-strategy)
11. [Testing Strategy](#11-testing-strategy)

**Part III: Implementation Plan**
12. [Conformance Checklist](#12-conformance-checklist)

---

## 1. Project Goal

Create a high-performance, generic, and extensible Rust crate (`tiles_tools`) that provides comprehensive tools for developing tile-based games and applications, following strict design and codestyle rulebooks while leveraging the wTools ecosystem.

## 2. Problem Solved

Game developers frequently reinvent fundamental grid and tilemap mechanics. `tiles_tools` provides a single, robust library with a versatile data model, grid topology support, pathfinding capabilities, and procedural generation, built on proven architectural patterns from the wTools ecosystem.

## 3. Ubiquitous Language

| Term | Definition |
|------|------------|
| **Grid** | Abstract topological and geometric space on which the game exists |
| **Primal Grid** | Primary grid structure (Faces, Edges, Vertices) |
| **Dual Grid** | Graph representation dual to the Primal Grid |
| **Coordinate** | Generic position representation with System and Orientation type parameters |
| **System** | Coordinate system type (Axial, Offset<Parity>) |
| **Orientation** | Grid orientation (Pointy, Flat for hexagonal grids) |
| **Parity** | Offset coordinate variant (Odd, Even) for rectangular storage |
| **Pixel** | 2D Cartesian coordinate for rendering and screen transformations |
| **Grid2D** | Storage container for tile data mapped to hexagonal coordinates |
| **Distance** | Trait for calculating grid-specific distances between coordinates |
| **Neighbors** | Trait for finding adjacent coordinates in a grid topology |
| **Pathfinding** | A* algorithm implementation for coordinate-generic path finding |
| **Geometry** | Mesh generation utilities for hexagonal shapes and transformations |

## 4. Deliverables

1. **Published Rust Crate**: Compliant with workspace dependency management
2. **Source Code**: Following strict codestyle and design rulebooks  
3. **Comprehensive Tests**: All in `tests/` directory with 100% compliance
4. **API Documentation**: Generated via `cargo doc` with complete coverage
5. **Integration Examples**: Demonstrating real-world usage patterns

## 5. Functional Requirements

### 5.1 Core Data Architecture

- **FR-A1**: Must use lightweight ECS (HECS) with abstraction layer
- **FR-A2**: Entity must be type-safe newtype: `pub struct Entity(hecs::Entity)`
- **FR-A3**: All data stored in Components following wTools patterns
- **FR-A4**: No restrictions on custom components (extensibility first)
- **FR-A5**: Standard built-in components using `former` for builders
- **FR-A6**: Component tagging system for primitive types
- **FR-A7**: Efficient query interface abstracting HECS queries

### 5.2 Grid Management (Rulebook Compliant)

- **FR-B1**: `GridBuilder` using `former` crate for configuration
- **FR-B2**: Entity creation for all primitives with proper error handling
- **FR-B3**: Standard components generated with proper validation
- **FR-B4**: Parent-child relationships using Component hierarchy
- **FR-B5**: Query API following noun-verb naming: `grid.neighbors_query()`
- **FR-B6**: Symmetrical dual grid queries
- **FR-B7**: Geometric utilities: `grid.distance_calculate()`, `grid.visibility_check()`

### 5.3 Coordinate Systems (Currently Implemented)

- **FR-C1**: ✅ Hexagonal coordinate systems (Axial, Offset<Odd>, Offset<Even>)
- **FR-C2**: ✅ Pointy-top and Flat-top orientations with type safety
- **FR-C3**: ✅ Pixel coordinate conversion utilities with precise transformations
- **FR-C4**: ✅ Generic Coordinate<System, Orientation> with phantom types
- **FR-C5**: ✅ Serde serialization support for all coordinate types
- **FR-C6**: ✅ Distance and Neighbors traits implemented for Axial coordinates
- **FR-C7**: ✅ Mathematical operations (Add, Sub, Mul, Div) for Axial coordinates
- **FR-C8**: ✅ Conversion utilities between coordinate systems (From/Into traits)
- **FR-C9**: ✅ Directional methods (up, down, left_up, etc.) for navigation

### 5.4 Additional Grid Systems (Future Implementation)

- **FR-D1**: Square/Rectangular coordinate systems with 4/8 connectivity
- **FR-D2**: Triangular coordinate systems with 12-neighbor connectivity
- **FR-D3**: Isometric coordinate systems with diamond visual representation
- **FR-D4**: Voronoi diagram support for irregular tessellations

### 5.5 Grid Storage & Management (Currently Implemented)

- **FR-E1**: ✅ Grid2D<System, Orientation, T> generic storage container
- **FR-E2**: ✅ Coordinate-based indexing with bounds checking
- **FR-E3**: ✅ Iterator support (iter, indexed_iter) for grid traversal
- **FR-E4**: ✅ Insert/remove/get operations for Option<T> grids
- **FR-E5**: ✅ Default value initialization and custom function initialization

### 5.6 Pathfinding & Analysis (Currently Implemented)

- **FR-F1**: ✅ Generic A* pathfinding algorithm for any coordinate type
- **FR-F2**: ✅ Configurable accessibility and cost functions
- **FR-F3**: ✅ Integration with Distance and Neighbors traits
- **FR-F4**: Integration with pathfinding crate for optimized algorithms

### 5.7 Geometry & Rendering (Currently Implemented)

- **FR-G1**: ✅ Hexagonal mesh generation utilities
- **FR-G2**: ✅ 2D triangle mesh creation from coordinate iterators
- **FR-G3**: ✅ Transformation matrix support for shape positioning
- **FR-G4**: ✅ Line generation between pixel coordinates
- **FR-G5**: ✅ Unit hexagon vertex generation and triangulation

### 5.8 Procedural Generation (Future Implementation)

- **FR-H1**: Wave Function Collapse solver with constraint systems
- **FR-H2**: Noise generation utilities for terrain
- **FR-H3**: Tileset definition with rule-based constraints
- **FR-H4**: Pre-built generation patterns and examples

## 6. Non-Functional Requirements

### 6.1 Performance
- **NFR-1**: GridBuilder (100x100 grid) < 250ms
- **NFR-2**: A* pathfinding < 5ms for typical scenarios
- **NFR-3**: Memory layout optimized for cache locality

### 6.2 Code Quality (Rulebook Compliance)
- **NFR-4**: 100% adherence to Design Rulebook patterns
- **NFR-5**: 100% adherence to Codestyle Rulebook formatting
- **NFR-6**: All file names use snake_case
- **NFR-7**: All functions use noun-verb naming order
- **NFR-8**: Use 2-space indentation throughout
- **NFR-9**: Explicit lifetime parameters where required

### 6.3 Documentation & Testing
- **NFR-10**: 100% documentation coverage via `cargo doc`
- **NFR-11**: All tests in `tests/` directory (no inline tests)
- **NFR-12**: Test Matrix planning for comprehensive coverage
- **NFR-13**: Integration test feature gating

### 6.4 Dependency Management
- **NFR-14**: All dependencies inherited from workspace
- **NFR-15**: Exclusive use of `error_tools` for error handling
- **NFR-16**: Use `former` for all builder patterns
- **NFR-17**: Proper `enabled` and `full` feature flags

## 7. Dependency Architecture

### 7.1 Required wTools Dependencies
```toml
[dependencies]
# Core wTools ecosystem
error_tools = { workspace = true }
former = { workspace = true }
mod_interface = { workspace = true }

# ECS and core functionality  
hecs = { workspace = true }
ndarray_cg = { workspace = true }
pathfinding = { workspace = true }
serde = { workspace = true, features = ["derive"] }

# Development and testing
test_tools = { workspace = true, optional = true }
```

### 7.2 Current Architecture (Implemented)
```
src/
├── lib.rs                 # Public API facade with feature gating
├── coordinates/
│   ├── mod.rs             # Distance and Neighbors trait definitions
│   ├── hexagonal.rs       # Coordinate<System, Orientation> with full impl
│   └── pixel.rs           # Pixel coordinate with conversions
├── collection.rs          # Grid2D generic storage container
├── pathfind.rs           # Generic A* pathfinding implementation  
├── geometry.rs           # Hexagonal mesh generation utilities
└── layout.rs             # Grid layout definitions (if enabled)
```

### 7.3 Future Architecture Extensions
```
src/
├── coordinates/
│   ├── square.rs          # Square grid coordinate systems
│   ├── triangular.rs      # Triangular grid coordinate systems
│   ├── isometric.rs       # Isometric coordinate transformations
│   └── voronoi.rs         # Irregular tessellation support
├── generation/
│   ├── wfc.rs            # Wave Function Collapse implementation
│   ├── noise.rs          # Terrain noise generation
│   └── constraints.rs    # Rule-based generation systems
├── analysis/
│   ├── flow_field.rs     # Multi-unit movement calculations
│   ├── visibility.rs     # Field of view and line-of-sight
│   └── regions.rs        # Flood fill and area analysis
└── ecs/
    ├── entity.rs         # ECS entity management
    ├── component.rs      # Standard tile components
    └── world.rs          # ECS world integration
```

## 8. Module Structure

Each module follows the strict mod_interface pattern:

```rust
// Example: src/core/mod.rs
mod private
{
  pub struct Entity(hecs::Entity);
  pub struct World(hecs::World);
  
  impl World
  {
    pub fn entity_create(&mut self) -> Entity { /* ... */ }
    pub fn component_add<C>(&mut self, entity: Entity, component: C) 
    -> error_tools::Result<()> { /* ... */ }
  }
}

crate::mod_interface!
{
  exposed use private::Entity;
  exposed use private::World;
}
```

## 9. Error Handling Strategy

Exclusive use of `error_tools` crate:

```rust
use error_tools::{ err, Result, BasicError };

#[derive(Debug)]
pub enum TilesError
{
  CoordinateOutOfBounds{ coord: String },
  PathfindingFailed{ from: String, to: String },
  InvalidTileset{ reason: String },
}

impl std::fmt::Display for TilesError
{
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
  {
    match self
    {
      TilesError::CoordinateOutOfBounds{ coord } =>
        write!(f, "Coordinate out of bounds: {}", coord),
      // ... other variants
    }
  }
}

impl std::error::Error for TilesError {}

// Usage
fn pathfinding_calculate(from: Coordinate, to: Coordinate) 
-> error_tools::Result<Vec<Coordinate>>
{
  // ... pathfinding logic
  Ok(path)
}
```

## 10. Code Generation Strategy

Using `former` for all builder patterns:

```rust
use former::Former;

#[derive(Former, Debug)]
pub struct GridConfig
{
  pub width: u32,
  pub height: u32,
  pub topology: GridTopology,
}

// Usage becomes:
let config = GridConfig::former()
  .width(100)
  .height(100)
  .topology(GridTopology::Hexagonal)
  .form();
```

## 11. Testing Strategy

### 11.1 Directory Structure
```
tests/
├── integration/
│   ├── grid_tests.rs
│   ├── pathfinding_tests.rs
│   └── generation_tests.rs
├── unit/
│   ├── coordinates_tests.rs
│   └── core_tests.rs
└── examples/
    ├── game_of_life.rs
    ├── chess.rs
    └── procedural_island.rs
```

### 11.2 Test Matrix Approach
Each test file includes a comprehensive test matrix:

```rust
//! ## Test Matrix for Grid Operations
//!
//! | ID   | Operation | Grid Type | Expected Result |
//! |------|-----------|-----------|-----------------|
//! | T1.1 | Create    | Hexagonal | Success         |
//! | T1.2 | Create    | Square    | Success         |
//! | T2.1 | Query     | Valid     | Returns data    |
//! | T2.2 | Query     | Invalid   | Returns error   |

/// Tests grid creation with hexagonal topology
/// Test Combination: T1.1
#[test]
fn test_grid_creation_hexagonal()
{
  let config = GridConfig::former()
    .width(10)
    .height(10)
    .topology(GridTopology::Hexagonal)
    .form();
    
  let result = Grid::builder_create(config);
  assert!(result.is_ok());
}
```

### 11.3 Feature Gating
```toml
[features]
default = ["enabled"]
enabled = ["dep:hecs", "dep:pathfinding", "dep:serde"]
full = ["enabled"] 
integration = []
```

```rust
// At top of each integration test file
#![cfg(feature = "integration")]
```

## 12. Conformance Checklist

### Design Rulebook Compliance
- ✅ **Lifetimes Explicit**: All function signatures use explicit lifetime parameters
- ✅ **No Arbitrary Reformatting**: Existing code style preserved, new code follows rulebook
- ✅ **Only Requested Features**: Implementation strictly follows specification
- ✅ **ECS Architecture**: Uses HECS with proper abstraction layer
- ✅ **Modular Design**: Traits for clear contracts and flexibility
- ✅ **Error Handling Centralized**: Exclusive use of `error_tools`
- ✅ **Testing Mandatory**: All code changes include comprehensive tests
- ✅ **Tests in Directory**: All tests in `tests/` directory, not `src/`

### Codestyle Rulebook Compliance  
- ✅ **Universal Applicability**: All Rust code follows codestyle rules
- ✅ **File Naming**: All files use snake_case lowercase
- ✅ **Entity Naming**: All functions follow noun-verb order
- ✅ **Indentation**: 2 spaces throughout, no tabs
- ✅ **Newlines for Blocks**: Opening braces on new lines
- ✅ **Spaces Around Symbols**: Proper spacing around operators
- ✅ **Module Structure**: Uses mod_interface pattern
- ✅ **Error Handling**: Exclusive use of error_tools
- ✅ **Workspace Dependencies**: All deps inherited from workspace
- ✅ **Feature Flags**: Proper enabled/full feature structure

### Current Implementation Status
- ✅ **Hexagonal Coordinates**: Full implementation with Axial/Offset systems
- ✅ **Type Safety**: Generic Coordinate<System, Orientation> design
- ✅ **Serde Support**: Serialization for all coordinate types  
- ✅ **Grid Storage**: Generic Grid2D container with coordinate indexing
- ✅ **Pathfinding**: Generic A* algorithm for any coordinate system
- ✅ **Geometry**: Hexagonal mesh generation and transformations
- ✅ **Pixel Conversion**: Accurate hex-to-pixel coordinate transformations
- ✅ **Distance/Neighbors**: Trait-based grid topology abstractions
- ✅ **Mathematical Ops**: Full arithmetic support for Axial coordinates
- ✅ **Documentation**: Comprehensive rustdoc with examples

### Pending Implementation
- ⏳ **ECS Integration**: HECS-based entity-component system
- ⏳ **Former Builders**: Configuration builders for complex structures
- ⏳ **Additional Grids**: Square, triangular, isometric coordinate systems
- ⏳ **Procedural Generation**: WFC and noise generation systems
- ⏳ **Advanced Analysis**: Flow fields, visibility, region analysis
- ⏳ **Performance Testing**: Benchmarking and optimization validation
- ⏳ **Integration Examples**: Real-world usage demonstrations

## Conclusion

This updated specification ensures full compliance with the Design and Codestyle Rulebooks while leveraging the wTools ecosystem effectively. The architecture prioritizes maintainability, type safety, and performance while providing a clean, extensible API for tile-based game development.
