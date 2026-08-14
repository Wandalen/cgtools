# Algorithm: Coordinate System Conversion

### Scope

- **Purpose**: Document `Convert`/`ApproximateConvert`, the exact-vs-lossy coordinate conversion traits, and which coordinate-system pairs each is implemented for.
- **Responsibility**: Document every `impl Convert`/`impl ApproximateConvert` pairing that exists, and the one coordinate system (`triangular`) with none.
- **In Scope**: `Convert<T>`, `ApproximateConvert<T>`, `BatchConvertExact`/`BatchConvertApproximate`, every concrete `impl` in `src/coordinates/conversion.rs`.
- **Out of Scope**: Conversion to/from `Pixel` (screen space) — that is per-coordinate-system `to_pixel`/`from_pixel` methods, not part of the `Convert`/`ApproximateConvert` trait pair (see `type/001`).

### Abstract

`tiles_tools::coordinates::conversion` splits inter-system conversion into two traits with different correctness guarantees: `Convert<T>` for lossless, exact conversions (used only where one grid is a literal reindexing of another with no approximation) and `ApproximateConvert<T>` for lossy conversions (used wherever the source and target topologies don't tile the plane the same way, e.g. hex-to-square). `BatchConvertExact`/`BatchConvertApproximate` extend both to `Vec<T>` for bulk conversion. Not every coordinate system pair — or even every coordinate system — has a conversion path.

### Algorithm

**Exact (`Convert`)** — implemented only between `square::Coordinate` and `isometric::Coordinate<Diamond>`, both directions, both `Connectivity` variants (`src/coordinates/conversion.rs:129-155`):
- `SquareCoord<SquareFour> ↔ IsoCoord<Diamond>`
- `SquareCoord<SquareEight> ↔ IsoCoord<Diamond>`

This pairing is exact because isometric coordinates are, by the crate's own design, a pure visual reprojection of square-grid logical coordinates (consistent with `algorithm/001`'s note that `Diamond`'s `Distance` impl explicitly reuses square-grid Manhattan distance) — no information is gained or lost converting between them.

**Approximate (`ApproximateConvert`)** — implemented for every other cross-topology pairing present in the crate (`src/coordinates/conversion.rs:166-215`):
- `hexagonal::Coordinate<Axial, Orientation> → square::Coordinate<SquareFour>`
- `hexagonal::Coordinate<Axial, Orientation> → square::Coordinate<SquareEight>`
- `square::Coordinate<Connectivity> → hexagonal::Coordinate<Axial, Pointy>`
- `hexagonal::Coordinate<Axial, Orientation> → isometric::Coordinate<Diamond>`
- `isometric::Coordinate<Diamond> → hexagonal::Coordinate<Axial, Pointy>`

Every pairing involving `hexagonal` is approximate — a hex grid's 6-neighbor topology has no exact correspondence to a square or diamond grid's 4/8-neighbor topology, so any such conversion necessarily discards or distorts adjacency information. `approximate_conversion_error_measure<T, U>` (`src/coordinates/conversion.rs:335+`) and `roundtrip_conversion_test<T, U, V>` (`src/coordinates/conversion.rs:319+`) exist specifically to quantify this loss for a given conversion pair.

**`triangular::Coordinate` has zero conversion implementations** — neither `Convert` nor `ApproximateConvert` is implemented in either direction between `triangular` and any other coordinate system. A caller needing to move a triangular-grid position into square, hex, or isometric space (e.g. to reuse `data_structure/002`'s `Quadtree`, whose `SpatialCoordinate` is not implemented for `triangular` either) must write that conversion by hand; there is no crate-provided starting point.

**Batch conversion** (`BatchConvertExact`/`BatchConvertApproximate`, `src/coordinates/conversion.rs:224-265`, plus the free functions `batch_convert_exact`/`batch_convert_approximate`) are generic wrappers applying the single-value trait element-wise over a `Vec<T>` — no batch-specific optimization (e.g. no shared-computation reuse across elements), just a `Vec` of the same per-element conversion.

### Types

| File | Relationship |
|------|--------------|
| [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) | Every coordinate type named in the pairings above is defined there; the same doc's Y-axis disclosure is most relevant exactly at this conversion boundary |

### Data Structures

| File | Relationship |
|------|--------------|
| [data_structure/002_spatial_quadtree.md](../data_structure/002_spatial_quadtree.md) | `triangular`'s lack of any conversion path compounds its lack of a `SpatialCoordinate` impl — there is no crate-provided route from a triangular position into quadtree-compatible space |

### Sources

| File | Relationship |
|------|--------------|
| `src/coordinates/conversion.rs` | `Convert`, `ApproximateConvert`, `BatchConvertExact`, `BatchConvertApproximate`, every `impl` listed above, `approximate_conversion_error_measure`, `roundtrip_conversion_test` |

### Tests

No dedicated regression test currently pins `triangular`'s absence from either conversion trait as intentional (vs. simply not yet implemented).
