# Type: Coordinate System Type Model

### Scope

- **Purpose**: Define the phantom-type pattern that gives each grid topology (hexagonal, square, triangular, isometric) its own compile-time-distinct coordinate type, plus the one type (`Pixel`) that deliberately sits outside this pattern.
- **Responsibility**: Document each `Coordinate<...>` type's phantom parameters, what compile-time guarantee the pattern buys, and where the pattern's coverage is incomplete (`Pixel`'s lack of a grid marker, its lack of `Serialize`/`Deserialize`, and a Y-axis convention conflict between `Pixel` and grid coordinates).
- **In Scope**: `hexagonal::Coordinate<System, Orientation>`, `square::Coordinate<Connectivity>`, `triangular::Coordinate<Orientation>`, `isometric::Coordinate<Projection>`, `pixel::Pixel` — their marker types and derived traits.
- **Out of Scope**: The sum-constraint runtime invariant on triangular coordinates (see `invariant/001`); distance/neighbor formulas (see `algorithm/001`); inter-system conversion (see `algorithm/005`); how these types are used as ECS component payloads (see `type/002`).

### Definition

Every grid coordinate type in `tiles_tools` is a thin wrapper around integer fields, made distinct per grid topology and per topology *variant* via zero-sized phantom marker types rather than via separate field layouts. This is a compile-time-only distinction — `PhantomData` markers add no runtime cost — that prevents, for example, passing a `square::Coordinate<FourConnected>` anywhere a `hexagonal::Coordinate<Axial, Pointy>` is expected, even though both are structurally "two i32s."

| Type | Phantom parameter(s) | Marker types | Source |
|------|----------------------|---------------|--------|
| `hexagonal::Coordinate<System, Orientation>` | `System` (storage scheme), `Orientation` (visual layout) | `Axial`, `Offset<Parity>` (`Parity` ∈ `Odd`/`Even`) × `Pointy`, `Flat` | `src/coordinates/hexagonal.rs:15-40` |
| `square::Coordinate<Connectivity>` | `Connectivity` (which neighbor set applies) | `FourConnected`, `EightConnected` | `src/coordinates/square.rs:41-62` |
| `triangular::Coordinate<Orientation>` | `Orientation` (which edge is the base) | `FlatTopped`, `FlatSided` | `src/coordinates/triangular.rs:8-17` |
| `isometric::Coordinate<Projection>` | `Projection` (screen-projection scheme) | `Diamond` | `src/coordinates/isometric.rs:56-85` |
| `pixel::Pixel` | none | none — plain `{ x: f32, y: f32 }` | `src/coordinates/pixel.rs:9-17` |

`hexagonal::Offset<Parity>` is itself a second-order phantom type — `Coordinate<Offset<Odd>, Pointy>` and `Coordinate<Offset<Even>, Pointy>` are distinct types, reflecting that odd/even row (or column) offset grids use different neighbor-adjacency arithmetic despite an identical `{q, r}` field layout.

**`Pixel` is the deliberate exception to the pattern.** It carries no phantom marker at all — it is not "the pixel representation of grid system X," it is grid-agnostic screen/world space, produced by any grid type's `to_pixel`/`to_screen` conversion (see `algorithm/005`) and consumed the same way regardless of which grid produced it. This is consistent with its role: once a coordinate has been projected to pixel space, which grid it came from is no longer a meaningful distinction for rendering.

### Validation

**What the phantom-type pattern enforces**: at compile time, a function generic over `C: Distance` or `C: Neighbors` (see `algorithm/001`) cannot be called with a coordinate from the wrong grid topology, and two coordinates from different marker instantiations (e.g. `Coordinate<Axial, Pointy>` vs. `Coordinate<Axial, Flat>`) cannot be compared, added, or substituted for each other — the type checker rejects it before the program runs. This is the pattern's entire value: zero runtime cost, compile-time-only grid-mixing prevention.

**What it does not enforce**:

- **`Pixel` carries no `Serialize`/`Deserialize`.** Every other coordinate type in the table above derives both (`hexagonal::Coordinate` and `square::Coordinate` via `#[derive(Serialize, Deserialize)]`; `triangular::Coordinate` via a hand-written `impl` pair validating the sum constraint on deserialize — see `invariant/001`; `isometric::Coordinate` via derive). `Pixel` derives only `Debug, Default, Clone, Copy, PartialEq` (`src/coordinates/pixel.rs:9`) — no `Serialize`/`Deserialize`. A generic component wrapping `Pixel` directly (e.g. a hypothetical `Position<Pixel>`) would not receive a derived `Serialize`/`Deserialize` impl from `#[derive]`'s auto-generated bounds the way `Position<C>` does for every other coordinate type (see `type/002`).
- **Two conflicting, independently-documented Y-axis conventions coexist.** `src/coordinates/pixel.rs:8`: *"It is assumed that the Y-axis points downwards"* (screen-space convention). `src/coordinates/square.rs:56`: *"y increases upward (mathematical convention)"* for `square::Coordinate`'s own axis. Both comments are deliberate, explicit design statements, not oversights in isolation — but nothing in the type system flags the boundary where a value crosses from one convention to the other (e.g. a `square::Coordinate` converted `to_pixel`/`to_screen`). A caller composing grid-space and pixel-space Y values directly (rather than through a provided conversion) can silently invert vertical direction.
- **The phantom-type pattern only prevents *cross-topology* mixing.** It says nothing about whether a given coordinate's *field values* are individually valid — that is a per-type, per-invariant question. `triangular::Coordinate` is the one type in this table with a real field-value constraint (see `invariant/001`); the other four types accept any `i32` pair/values their constructors are given.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/001_coordinate_distance_and_neighbor_formulas.md](../algorithm/001_coordinate_distance_and_neighbor_formulas.md) | `Distance`/`Neighbors` trait impls operate per-type on the marker combinations listed above |
| [algorithm/002_generic_astar_pathfinding.md](../algorithm/002_generic_astar_pathfinding.md) | `C: Distance + Neighbors` bound is satisfied by any type in this table |
| [algorithm/004_hexagon_geometry_generation.md](../algorithm/004_hexagon_geometry_generation.md) | Not a direct dependency — these generators take no coordinate-type parameter from this table; positioning with any type here is entirely the caller's responsibility |
| [algorithm/005_coordinate_system_conversion.md](../algorithm/005_coordinate_system_conversion.md) | `Convert`/`ApproximateConvert` move values between these types (and to/from `Pixel`); the Y-axis conflict above is most visible at this boundary |

### Data Structures

| File | Relationship |
|------|--------------|
| [data_structure/001_grid2d_dense_hex_bounded_storage.md](../data_structure/001_grid2d_dense_hex_bounded_storage.md) | `Grid2D`'s `System`/`Orientation` parameters select among this table's hexagonal marker combinations only — not usable with any non-hexagonal type here |
| [data_structure/002_spatial_quadtree.md](../data_structure/002_spatial_quadtree.md) | `SpatialCoordinate`'s two implementors ((i32, i32), `square::Coordinate<T>`) are a small subset of this table's full coordinate-type list |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_triangular_coordinate_sum_constraint.md](../invariant/001_triangular_coordinate_sum_constraint.md) | The one field-value constraint among these five types, and how it interacts with the type's own constructors |
| [invariant/002_lattice_address_primacy.md](../invariant/002_lattice_address_primacy.md) | The phantom-typed coordinate families defined here are the typed lattice addresses that invariant requires as the sole storage authority |

### Types

| File | Relationship |
|------|--------------|
| [type/002_ecs_component_vocabulary.md](../type/002_ecs_component_vocabulary.md) | `Position<C>` is generic over any type in this table; `C`'s `Serialize`/`Deserialize` availability (or lack of it, for `Pixel`) propagates directly into `Position<C>`'s own derived impls |

### Sources

| File | Relationship |
|------|--------------|
| `src/coordinates.rs` | `Distance`, `Neighbors`, `ToDual` trait definitions; unconditional `pub mod` declarations for all five submodules |
| `src/coordinates/hexagonal.rs` | `Axial`, `Offset<Parity>`, `Odd`, `Even`, `Pointy`, `Flat`, `Coordinate<System, Orientation>` |
| `src/coordinates/square.rs` | `FourConnected`, `EightConnected`, `Coordinate<Connectivity>` |
| `src/coordinates/triangular.rs` | `FlatTopped`, `FlatSided`, `Coordinate<Orientation>` |
| `src/coordinates/isometric.rs` | `Diamond`, `Coordinate<Projection>` |
| `src/coordinates/pixel.rs` | `Pixel` |

### Tests

No dedicated regression test currently pins the Y-axis convention or `Pixel`'s missing `Serialize`/`Deserialize` as intentional — both are documented-in-source design statements (see the quoted comments above) rather than behavior covered by an assertion.
