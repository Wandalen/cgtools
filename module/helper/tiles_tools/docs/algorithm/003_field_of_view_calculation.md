# Algorithm: Field of View Calculation

### Scope

- **Purpose**: Document `FieldOfView`'s four selectable algorithms and the visibility/lighting types built around them.
- **Responsibility**: Document `FOVAlgorithm`'s variants, `VisibilityMap<C>`/`VisibilityState`, and `LightSource<C>`/`LightingCalculator<C>`.
- **In Scope**: `FOVAlgorithm` (`Shadowcasting`, `RayCasting`, `FloodFill`, `Bresenham`), `VisibilityState`, `VisibilityMap<C>`, `FieldOfView`, `LightSource<C>`, `LightingCalculator<C>`.
- **Out of Scope**: Spatial range queries feeding candidate positions into FOV (see `data_structure/002`); the coordinate `Distance`/`Neighbors` formulas FOV calculations are generic over (see `algorithm/001`).

### Abstract

`tiles_tools::field_of_view` computes, from a viewer position, which grid cells are currently visible, tracks a per-cell `VisibilityState` (distinguishing "currently visible" from "previously seen, now out of sight" — the standard fog-of-war distinction), and layers a separate `LightSource`/`LightingCalculator` pair on top for illumination rather than pure sight-line visibility. `FOVAlgorithm` names four selectable techniques; there is no separate `VisionCalculator` type — `FieldOfView` itself is the entry point that dispatches on the selected `FOVAlgorithm` variant.

### Algorithm

`FOVAlgorithm` (`src/field_of_view.rs:46-55`) offers four named variants, each with a doc-comment-stated performance/quality tradeoff:

| Variant | Doc-stated tradeoff |
|---------|----------------------|
| `Shadowcasting` | "balanced speed/quality" |
| `RayCasting` | "slower but precise" |
| `FloodFill` | "fast for small ranges" |
| `Bresenham` | "fast but basic" |

`VisibilityMap<C>` (`src/field_of_view.rs:112+`) stores a `VisibilityState` per coordinate visited by the last calculation; `VisibilityState` (`src/field_of_view.rs:60+`) distinguishes currently-visible cells from remembered-but-not-currently-visible ones, the mechanism a caller would build fog-of-war on top of. `FieldOfView` (`src/field_of_view.rs:216+`) is the calculation entry point taking a viewer position, range, and `FOVAlgorithm` selection.

`LightSource<C>`/`LightingCalculator<C>` (`src/field_of_view.rs:784+`, `829+`) are a separate concern layered alongside visibility rather than derived from it — a lit-but-not-directly-visible cell and a visible-but-unlit cell are both representable.

Each `FOVAlgorithm` variant's fidelity to the textbook technique its name references (e.g. whether `Shadowcasting` implements genuine recursive shadowcasting with per-octant slope tracking, versus a simpler approximation using the same name) was not re-verified line-by-line as part of this migration — a caller whose gameplay depends on a specific algorithm's precise visibility shape (not just "some reasonable FOV") should read the corresponding implementation in `src/field_of_view.rs` directly before relying on the name alone.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/001_coordinate_distance_and_neighbor_formulas.md](../algorithm/001_coordinate_distance_and_neighbor_formulas.md) | Range-limited FOV variants are generic over the same `Distance`/`Neighbors` traits |

### Data Structures

| File | Relationship |
|------|--------------|
| [data_structure/002_spatial_quadtree.md](../data_structure/002_spatial_quadtree.md) | A caller narrowing FOV computation to nearby entities would typically source candidates from a spatial query first |

### Sources

| File | Relationship |
|------|--------------|
| `src/field_of_view.rs` | `FOVAlgorithm`, `VisibilityState`, `VisibilityMap<C>`, `FieldOfView`, `LightSource<C>`, `LightingCalculator<C>` |

### Tests

Coverage was not individually re-verified per `FOVAlgorithm` variant as part of this migration; see `src/field_of_view.rs`'s own `#[cfg(test)]` module for current test presence per algorithm.
