# Tiles Tools Engine Specification

**Name:** Tiles Tools Engine  
**Version:** 1.1 (Rulebook Compliant)  
**Date:** 2025-08-08

## Table of Contents

**Part I: Public Contract (Mandatory Requirements)**
1. [Project Goal](#1-project-goal)
2. [Problem Solved](#2-problem-solved)
3. [Ubiquitous Language](#3-ubiquitous-language)
4. [Use Cases & Applications](#4-use-cases--applications)
5. [Deliverables](#5-deliverables)
6. [Functional Requirements](#6-functional-requirements)
7. [Non-Functional Requirements](#7-non-functional-requirements)
8. [Dependency Architecture](#8-dependency-architecture)

**Part II: Internal Design (Rulebook Compliant)**
9. [Module Structure](#9-module-structure)
10. [Error Handling Strategy](#10-error-handling-strategy)
11. [Code Generation Strategy](#11-code-generation-strategy)
12. [Testing Strategy](#12-testing-strategy)

**Part III: Implementation Plan**
13. [Conformance Checklist](#13-conformance-checklist)

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

## 4. Use Cases & Applications

### 4.1 Primary Use Cases

| Use Case | Grid Type | Key Features | Complexity |
|----------|-----------|--------------|------------|
| **Turn-Based Strategy Games** | Hexagonal | Pathfinding, territory control, equal movement | Medium |
| **Classic RPGs & Roguelikes** | Square | Dungeon layouts, simple movement, item placement | Low |
| **City Builders & Simulations** | Square/Isometric | Large grids, entity management, zoning | High |
| **Tactical RPGs** | Hexagonal | FOV, range calculations, positioning tactics | High |
| **Puzzle Games** | Square/Triangular | Grid manipulation, pattern matching | Medium |
| **Board Game Adaptations** | Hexagonal/Square | Rule enforcement, turn management | Medium |
| **Procedural Map Generation** | Any | Terrain generation, WFC, noise-based | High |
| **Real-Time Strategy** | Square/Hex | Flow fields, multi-unit pathfinding | Very High |

### 4.2 Concrete Usage Examples

#### Example 1: Tactical RPG Movement System
```rust
use tiles_tools::{
    coordinates::hexagonal::{Coordinate, Axial, Pointy},
    pathfind::astar,
    coordinates::{Distance, Neighbors},
    collection::Grid2D,
};

// Character movement in tactical RPG
struct Character { movement_range: u32, position: HexCoord }
type HexCoord = Coordinate<Axial, Pointy>;

fn valid_moves(character: &Character, obstacles: &Grid2D<Axial, Pointy, bool>) 
    -> Vec<HexCoord> 
{
    let mut valid_positions = Vec::new();
    
    // Check all positions within movement range
    for candidate in hex_spiral_range(character.position, character.movement_range) {
        if let Some((path, cost)) = astar(
            &character.position,
            &candidate,
            |pos| !obstacles.get(*pos).unwrap_or(&true), // Not blocked
            |_| 1, // Unit movement cost
        ) {
            if cost <= character.movement_range {
                valid_positions.push(candidate);
            }
        }
    }
    valid_positions
}
```

#### Example 2: Procedural Dungeon Generation
```rust
use tiles_tools::{
    coordinates::square::{Coordinate, FourConnected},
    collection::Grid2D,
    generation::wfc::WaveFunction, // Future implementation
};

type SquareCoord = Coordinate<FourConnected>;

#[derive(Clone, Copy)]
enum TileType { Wall, Floor, Door, Treasure }

struct DungeonGenerator {
    grid: Grid2D<FourConnected, Option<TileType>>,
    rules: TilesetRules,
}

impl DungeonGenerator {
    fn generate_room(&mut self, top_left: SquareCoord, width: u32, height: u32) {
        // Create room boundaries
        for x in 0..width {
            for y in 0..height {
                let pos = SquareCoord::new(top_left.x + x as i32, top_left.y + y as i32);
                let tile = if x == 0 || x == width-1 || y == 0 || y == height-1 {
                    TileType::Wall
                } else {
                    TileType::Floor
                };
                self.grid.insert(pos, tile);
            }
        }
    }
    
    fn connect_rooms(&mut self, room1: SquareCoord, room2: SquareCoord) {
        if let Some((path, _)) = astar(&room1, &room2, |_| true, |_| 1) {
            for pos in path {
                self.grid.insert(pos, TileType::Floor);
            }
        }
    }
}
```

#### Example 3: City Builder Zoning System
```rust
use tiles_tools::{
    coordinates::isometric::{Coordinate, Isometric},
    analysis::{RegionAnalyzer, FlowField}, // Future implementation
    ecs::{World, Entity}, // Future implementation
};

type IsoCoord = Coordinate<Isometric>;

enum ZoneType { Residential, Commercial, Industrial, Road }

struct CityPlanner {
    world: World,
    zones: Grid2D<Isometric, Option<ZoneType>>,
    traffic_flow: FlowField<Isometric>,
}

impl CityPlanner {
    fn calculate_land_value(&self, position: IsoCoord) -> f32 {
        let mut value = 100.0;
        
        // Distance to commercial zones increases residential value
        if let Some(commercial_distance) = self.nearest_zone_distance(position, ZoneType::Commercial) {
            value += (10.0 / (commercial_distance as f32 + 1.0)) * 50.0;
        }
        
        // Distance to industrial zones decreases residential value
        if let Some(industrial_distance) = self.nearest_zone_distance(position, ZoneType::Industrial) {
            value -= (10.0 / (industrial_distance as f32 + 1.0)) * 30.0;
        }
        
        // Traffic accessibility increases commercial value
        value += self.traffic_flow.accessibility_at(position) * 25.0;
        
        value
    }
}
```

#### Example 4: Real-Time Strategy Unit Management
```rust
use tiles_tools::{
    coordinates::square::{Coordinate, EightConnected},
    analysis::FlowField,
    pathfind::flow_field_calculate, // Future implementation
};

type RTSCoord = Coordinate<EightConnected>;

struct Unit {
    id: u32,
    position: RTSCoord,
    target: Option<RTSCoord>,
    unit_type: UnitType,
}

enum UnitType { Infantry, Tank, Aircraft }

struct RTSGameState {
    units: Vec<Unit>,
    obstacles: Grid2D<EightConnected, bool>,
    flow_fields: HashMap<RTSCoord, FlowField<EightConnected>>,
}

impl RTSGameState {
    fn move_units_to_target(&mut self, target: RTSCoord) {
        // Calculate flow field for target position
        let flow_field = flow_field_calculate(
            target,
            |pos| !self.obstacles[*pos], // Passable terrain
            |_| 1, // Unit cost
        );
        
        // Move all units towards target using flow field
        for unit in &mut self.units {
            if let Some(next_position) = flow_field.best_move_from(unit.position) {
                unit.position = next_position;
            }
        }
    }
    
    fn group_move(&mut self, units: &[u32], formation: Formation, target: RTSCoord) {
        // Calculate formation positions around target
        let formation_positions = formation.calculate_positions(target, units.len());
        
        // Assign each unit to its formation position
        for (unit_id, formation_pos) in units.iter().zip(formation_positions) {
            if let Some(unit) = self.units.iter_mut().find(|u| u.id == *unit_id) {
                unit.target = Some(formation_pos);
            }
        }
    }
}
```

### 4.3 Performance Characteristics

| Operation | Hexagonal | Square | Triangular | Complexity |
|-----------|-----------|---------|------------|------------|
| **Coordinate Creation** | O(1) | O(1) | O(1) | Constant |
| **Distance Calculation** | O(1) | O(1) | O(1) | Constant |
| **Neighbor Finding** | O(1) → 6 | O(1) → 4/8 | O(1) → 12 | Constant |
| **A\* Pathfinding** | O(b^d) | O(b^d) | O(b^d) | Exponential |
| **Flow Field Generation** | O(n) | O(n) | O(n) | Linear |
| **Grid Storage Access** | O(1) | O(1) | O(1) | Constant |
| **Coordinate Conversion** | O(1) | O(1) | O(1) | Constant |

**Legend:** n = grid size, b = branching factor, d = depth

## 5. Deliverables

1. **Published Rust Crate**: Compliant with workspace dependency management
2. **Source Code**: Following strict codestyle and design rulebooks  
3. **Comprehensive Tests**: All in `tests/` directory with 100% compliance
4. **API Documentation**: Generated via `cargo doc` with complete coverage
5. **Integration Examples**: Demonstrating real-world usage patterns
6. **Performance Benchmarks**: Criterion-based benchmarking suite
7. **Usage Tutorials**: Step-by-step guides for common use cases

## 6. Functional Requirements

### 6.1 Core Data Architecture

- **FR-A1**: Must use lightweight ECS (HECS) with abstraction layer
- **FR-A2**: Entity must be type-safe newtype: `pub struct Entity(hecs::Entity)`
- **FR-A3**: All data stored in Components following wTools patterns
- **FR-A4**: No restrictions on custom components (extensibility first)
- **FR-A5**: Standard built-in components using `former` for builders
- **FR-A6**: Component tagging system for primitive types
- **FR-A7**: Efficient query interface abstracting HECS queries

### 6.2 Grid Management (Rulebook Compliant)

- **FR-B1**: `GridBuilder` using `former` crate for configuration
- **FR-B2**: Entity creation for all primitives with proper error handling
- **FR-B3**: Standard components generated with proper validation
- **FR-B4**: Parent-child relationships using Component hierarchy
- **FR-B5**: Query API following noun-verb naming: `grid.neighbors_query()`
- **FR-B6**: Symmetrical dual grid queries
- **FR-B7**: Geometric utilities: `grid.distance_calculate()`, `grid.visibility_check()`

### 6.3 Coordinate Systems (Currently Implemented)

- **FR-C1**: ✅ Hexagonal coordinate systems (Axial, Offset<Odd>, Offset<Even>)
- **FR-C2**: ✅ Pointy-top and Flat-top orientations with type safety
- **FR-C3**: ✅ Pixel coordinate conversion utilities with precise transformations
- **FR-C4**: ✅ Generic Coordinate<System, Orientation> with phantom types
- **FR-C5**: ✅ Serde serialization support for all coordinate types
- **FR-C6**: ✅ Distance and Neighbors traits implemented for Axial coordinates
- **FR-C7**: ✅ Mathematical operations (Add, Sub, Mul, Div) for Axial coordinates
- **FR-C8**: ✅ Conversion utilities between coordinate systems (From/Into traits)
- **FR-C9**: ✅ Directional methods (up, down, left_up, etc.) for navigation

### 6.4 Additional Grid Systems (Future Implementation)

- **FR-D1**: Square/Rectangular coordinate systems with 4/8 connectivity
- **FR-D2**: Triangular coordinate systems with 12-neighbor connectivity
- **FR-D3**: Isometric coordinate systems with diamond visual representation
- **FR-D4**: Voronoi diagram support for irregular tessellations

### 6.5 Grid Storage & Management (Currently Implemented)

- **FR-E1**: ✅ Grid2D<System, Orientation, T> generic storage container
- **FR-E2**: ✅ Coordinate-based indexing with bounds checking
- **FR-E3**: ✅ Iterator support (iter, indexed_iter) for grid traversal
- **FR-E4**: ✅ Insert/remove/get operations for Option<T> grids
- **FR-E5**: ✅ Default value initialization and custom function initialization

### 6.6 Pathfinding & Analysis (Currently Implemented)

- **FR-F1**: ✅ Generic A* pathfinding algorithm for any coordinate type
- **FR-F2**: ✅ Configurable accessibility and cost functions
- **FR-F3**: ✅ Integration with Distance and Neighbors traits
- **FR-F4**: Integration with pathfinding crate for optimized algorithms

### 6.7 Geometry & Rendering (Currently Implemented)

- **FR-G1**: ✅ Hexagonal mesh generation utilities
- **FR-G2**: ✅ 2D triangle mesh creation from coordinate iterators
- **FR-G3**: ✅ Transformation matrix support for shape positioning
- **FR-G4**: ✅ Line generation between pixel coordinates
- **FR-G5**: ✅ Unit hexagon vertex generation and triangulation

### 6.8 Procedural Generation (Future Implementation)

- **FR-H1**: Wave Function Collapse solver with constraint systems
- **FR-H2**: Noise generation utilities for terrain
- **FR-H3**: Tileset definition with rule-based constraints
- **FR-H4**: Pre-built generation patterns and examples

## 7. Non-Functional Requirements

### 7.1 Performance
- **NFR-1**: GridBuilder (100x100 grid) < 250ms
- **NFR-2**: A* pathfinding < 5ms for typical scenarios
- **NFR-3**: Memory layout optimized for cache locality

### 7.2 Code Quality (Rulebook Compliance)
- **NFR-4**: 100% adherence to Design Rulebook patterns
- **NFR-5**: 100% adherence to Codestyle Rulebook formatting
- **NFR-6**: All file names use snake_case
- **NFR-7**: All functions use noun-verb naming order
- **NFR-8**: Use 2-space indentation throughout
- **NFR-9**: Explicit lifetime parameters where required

### 7.3 Documentation & Testing
- **NFR-10**: 100% documentation coverage via `cargo doc`
- **NFR-11**: All tests in `tests/` directory (no inline tests)
- **NFR-12**: Test Matrix planning for comprehensive coverage
- **NFR-13**: Integration test feature gating

### 7.4 Dependency Management
- **NFR-14**: All dependencies inherited from workspace
- **NFR-15**: Exclusive use of `error_tools` for error handling
- **NFR-16**: Use `former` for all builder patterns
- **NFR-17**: Proper `enabled` and `full` feature flags

## 8. Dependency Architecture

### 8.1 Required wTools Dependencies
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

### 8.2 Current Architecture (Implemented)
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

### 8.3 Future Architecture Extensions
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

## 9. Module Structure

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

## 10. Error Handling Strategy

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

## 11. Code Generation Strategy

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

## 12. Testing Strategy

### 12.1 Directory Structure
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

### 12.2 Test Matrix Approach
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

### 12.3 Feature Gating
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

## 13. Conformance Checklist

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

## Real-World Integration Examples

### Example Game Architectures

#### Hexagonal Tactical RPG (Comprehensive)
```rust
// Complete tactical RPG setup
use tiles_tools::{
    coordinates::hexagonal::{Coordinate, Axial, Pointy},
    collection::Grid2D,
    pathfind::astar,
    geometry::hexagon_triangles,
    ecs::{World, Entity}, // Future
};

type HexCoord = Coordinate<Axial, Pointy>;

struct TacticalRPG {
    world: World,
    terrain: Grid2D<Axial, Pointy, TerrainType>,
    units: Grid2D<Axial, Pointy, Option<Entity>>,
    current_player: PlayerId,
}

#[derive(Clone, Copy)]
enum TerrainType {
    Grass { movement_cost: u32 },
    Forest { movement_cost: u32, cover_bonus: i32 },
    Water { passable: bool },
    Mountain { height: u32 },
}

struct Unit {
    health: u32,
    movement_range: u32,
    attack_range: u32,
    position: HexCoord,
    owner: PlayerId,
}

impl TacticalRPG {
    fn calculate_attack_targets(&self, attacker: HexCoord, range: u32) -> Vec<HexCoord> {
        hex_ring_range(attacker, 1, range)
            .into_iter()
            .filter(|pos| self.units.get(*pos).is_some())
            .collect()
    }
    
    fn execute_turn(&mut self, unit_pos: HexCoord, action: Action) {
        match action {
            Action::Move(target) => {
                if let Some((path, cost)) = self.calculate_path(unit_pos, target) {
                    self.move_unit(unit_pos, target, path);
                }
            },
            Action::Attack(target) => {
                self.resolve_combat(unit_pos, target);
            },
            Action::Wait => {},
        }
    }
}
```

#### City Builder with Multiple Grid Types
```rust
// Multi-grid city builder
use tiles_tools::{
    coordinates::{square::*, isometric::*},
    collection::Grid2D,
    analysis::RegionAnalyzer, // Future
};

struct CityBuilder {
    // Logical grid for zoning and simulation
    logical_grid: Grid2D<FourConnected, ZoneData>,
    
    // Visual grid for rendering (isometric)
    visual_grid: Grid2D<Isometric, RenderData>,
    
    // Utility networks (separate grids)
    power_grid: Grid2D<FourConnected, PowerNode>,
    water_grid: Grid2D<FourConnected, WaterNode>,
    transport_grid: Grid2D<FourConnected, RoadNode>,
}

struct ZoneData {
    zone_type: ZoneType,
    development_level: u8,
    population: u32,
    happiness: f32,
}

impl CityBuilder {
    fn simulate_growth(&mut self) {
        for (coord, zone) in self.logical_grid.indexed_iter() {
            let growth_rate = self.calculate_growth_factors(coord);
            zone.population = (zone.population as f32 * growth_rate) as u32;
        }
    }
    
    fn update_visual_representation(&mut self) {
        for (logical_coord, zone_data) in self.logical_grid.indexed_iter() {
            let iso_coord = self.logical_to_isometric(logical_coord);
            let render_data = self.create_render_data(zone_data);
            self.visual_grid.insert(iso_coord, render_data);
        }
    }
}
```

#### Roguelike with Procedural Generation
```rust
// Comprehensive roguelike implementation
use tiles_tools::{
    coordinates::square::{Coordinate, FourConnected},
    collection::Grid2D,
    generation::{wfc::*, noise::*}, // Future
    analysis::{VisionCalculator, RegionAnalyzer}, // Future
};

type DungeonCoord = Coordinate<FourConnected>;

struct Roguelike {
    current_level: u32,
    dungeon: Grid2D<FourConnected, Tile>,
    entities: Grid2D<FourConnected, Option<Entity>>,
    player_pos: DungeonCoord,
    visible_tiles: HashSet<DungeonCoord>,
    explored_tiles: HashSet<DungeonCoord>,
}

#[derive(Clone, Copy)]
struct Tile {
    tile_type: TileType,
    blocks_movement: bool,
    blocks_sight: bool,
}

impl Roguelike {
    fn generate_level(&mut self, level: u32) {
        // Use WFC for coherent dungeon generation
        let tileset = load_dungeon_tileset(level);
        let generator = WaveFunction::new(100, 100, tileset);
        
        if let Ok(solution) = generator.solve() {
            self.dungeon = solution.to_grid();
        }
        
        // Add noise-based details
        self.add_environmental_details();
        
        // Place player and entities
        self.place_player_and_entities();
    }
    
    fn update_visibility(&mut self) {
        let vision_calculator = VisionCalculator::new(&self.dungeon);
        self.visible_tiles = vision_calculator.calculate_fov(
            self.player_pos,
            8, // Vision range
            |tile| !tile.blocks_sight,
        );
        
        // Add newly visible tiles to explored set
        self.explored_tiles.extend(&self.visible_tiles);
    }
    
    fn handle_player_input(&mut self, input: Input) {
        match input {
            Input::Move(direction) => {
                let new_pos = self.player_pos + direction.to_coord();
                if self.is_passable(new_pos) {
                    self.player_pos = new_pos;
                    self.update_visibility();
                }
            },
            Input::Attack(direction) => {
                let target_pos = self.player_pos + direction.to_coord();
                if let Some(entity) = self.entities.get(target_pos) {
                    self.resolve_combat(self.player_entity(), *entity);
                }
            },
        }
    }
}
```

---

This specification ensures full compliance with Design and Codestyle Rulebooks while demonstrating practical applications. The architecture prioritizes maintainability, type safety, and performance while providing a comprehensive toolkit for tile-based game development across multiple genres and complexity levels.
