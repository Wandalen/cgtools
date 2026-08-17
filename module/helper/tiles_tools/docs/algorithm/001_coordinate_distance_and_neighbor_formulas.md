# Algorithm: Coordinate Distance & Neighbor Formulas

### Scope

- **Purpose**: Document the per-coordinate-system `Distance`/`Neighbors` formulas, and the one API wrinkle where hexagonal `Axial` carries two same-named `distance` methods with different signatures.
- **Responsibility**: State each system's exact distance formula and neighbor offset set, and disclose the inherent-vs-trait `distance()` duplication on `hexagonal::Coordinate<Axial, _>`.
- **In Scope**: `Distance`/`Neighbors` impls for `hexagonal::Coordinate<Axial, _>`, `square::Coordinate<FourConnected|EightConnected>`, `triangular::Coordinate<_>`, `isometric::Coordinate<Diamond>`.
- **Out of Scope**: The triangular sum-constraint these formulas must preserve (see `invariant/001`); inter-system conversion (see `algorithm/005`); A*, which consumes these traits as generic bounds (see `algorithm/002`).

### Abstract

`Distance` and `Neighbors` (defined in `src/coordinates.rs` as `fn distance(&self, other: &Self) -> u32` and `fn neighbors(&self) -> Vec<Self>`) are the two traits every pathfinding- or range-aware algorithm in `tiles_tools` is generic over (see `algorithm/002`, `algorithm/003`). Each coordinate system implements both with a formula suited to its own topology — Manhattan, Chebyshev, or hex cube-distance — and `Neighbors` returns the fixed offset set that topology's adjacency implies (4, 6, or 8 neighbors). One implementor, `hexagonal::Coordinate<Axial, Orientation>`, additionally carries an inherent `distance` method distinct from its `Distance` trait implementation — documented in full below since the two are easy to invoke unintentionally interchangeably despite returning different types.

### Algorithm

**Hexagonal (`Axial`)** — cube-coordinate distance, the standard technique for hex grids (derive a third implicit axis `s = -q - r`, then `(|Δq| + |Δr| + |Δs|) / 2`):

```
s = -self.q - self.r;  other_s = -other.q - other.r;
Δq = self.q - other.q; Δr = self.r - other.r; Δs = s - other_s;
distance = (|Δq| + |Δr| + |Δs|) / 2
```

**Two implementations of this same formula coexist on `Coordinate<Axial, Orientation>`, with different signatures**:

| | Inherent method (`src/coordinates/hexagonal.rs:156-164`) | `Distance` trait impl (`src/coordinates/hexagonal.rs:390-402`) |
|---|---|---|
| Signature | `fn distance(&self, other: Self) -> i32` | `fn distance(&self, other: &Self) -> u32` |
| Parameter | **by value** | **by reference** |
| Return type | **signed `i32`** | **unsigned `u32`** |
| Arithmetic | `i32` throughout | widens to `i64` before `.abs()`, then narrows to `u32` |

Both compute the identical cube-distance formula and are mathematically equivalent for any two real coordinates (distance is always non-negative, so the `i32`/`u32` results agree numerically) — the divergence is in the *call shape*, not the *math*. `coord.distance(other)` (moving/copying `other` by value) resolves to the inherent method and yields `i32`; `coord.distance(&other)` resolves to the trait method (requires `Distance` in scope) and yields `u32`. Any generic code written against `C: Distance` only ever sees the trait version — the inherent method is reachable solely through direct, concrete `Coordinate<Axial, Orientation>` method-call syntax, never through a `Distance`-bounded generic parameter.

**Hexagonal neighbors (`Axial`)** — the 6 unit offsets in axial space: `(1,0), (1,-1), (0,-1), (-1,0), (-1,1), (0,1)` (`src/coordinates/hexagonal.rs:409-416`).

**Square** — two distance formulas selected by the `Connectivity` phantom parameter (see `type/001`):
- `FourConnected`: Manhattan distance, `|Δx| + |Δy|` (`src/coordinates/square.rs:183`) — matches its 4-neighbor (orthogonal-only) adjacency.
- `EightConnected`: Chebyshev distance, `max(|Δx|, |Δy|)` (`src/coordinates/square.rs:207`) — matches its 8-neighbor (orthogonal + diagonal) adjacency; a diagonal step costs the same as an orthogonal one under this metric, consistent with 8-directional movement.

**Triangular** — sum of absolute component differences across all three coordinates: `|Δa| + |Δb| + |Δc|` (`src/coordinates/triangular.rs:205-213`). Neighbors: exactly 3 (one per edge), computed as `new_unchecked(a ± 1, b, c)` / etc. with the sign chosen by current orientation — see `invariant/001` for why this derivation is guaranteed sum-preserving.

**Isometric (`Diamond`)** — reuses Manhattan distance over the underlying logical `(x, y)` pair (`src/coordinates/isometric.rs:271-274`), with the doc comment explicitly justifying this as inherited from the square grid isometric coordinates are a visual transform of (consistent with `algorithm/005`'s `Convert` impl between `Square` and `Diamond` being exact/lossless, not approximate).

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/002_generic_astar_pathfinding.md](../algorithm/002_generic_astar_pathfinding.md) | A* is generic over `C: Distance + Neighbors`, using these exact formulas for heuristic and expansion |
| [algorithm/003_field_of_view_calculation.md](../algorithm/003_field_of_view_calculation.md) | Range-limited FOV variants are generic over these same `Distance`/`Neighbors` formulas |
| [algorithm/005_coordinate_system_conversion.md](../algorithm/005_coordinate_system_conversion.md) | Isometric's Manhattan-distance reuse mirrors its exact `Convert` relationship with `Square` |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_triangular_coordinate_sum_constraint.md](../invariant/001_triangular_coordinate_sum_constraint.md) | Triangular `neighbors()`'s offset derivation is proven sum-preserving there |

### Pitfalls

| File | Relationship |
|------|--------------|
| [pitfall/004_hexagonal_axial_distance_method_ambiguity.md](../pitfall/004_hexagonal_axial_distance_method_ambiguity.md) | The inherent-vs-trait `distance()` duplication on `Coordinate<Axial, Orientation>` documented above is the concrete trap this pitfall covers |

### Types

| File | Relationship |
|------|--------------|
| [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) | Every coordinate type this algorithm operates on is defined there |

### Sources

| File | Relationship |
|------|--------------|
| `src/coordinates.rs` | `Distance`, `Neighbors` trait definitions |
| `src/coordinates/hexagonal.rs` | Both `distance` methods on `Coordinate<Axial, Orientation>`; `Neighbors` impl |
| `src/coordinates/square.rs` | `Distance`/`Neighbors` for `FourConnected`/`EightConnected` |
| `src/coordinates/triangular.rs` | `Distance`/`Neighbors` for `Coordinate<Orientation>` |
| `src/coordinates/isometric.rs` | `Distance`/`Neighbors` for `Coordinate<Diamond>` |

### Tests

| File | Relationship |
|------|--------------|
| `src/coordinates/square.rs` | Doc-tests inline on both `Distance` impls (`distance(&coord2) == 7`/`== 4`) |
| `src/coordinates/isometric.rs` | Doc-test inline on `Distance` impl (`distance == 7`) |

No test currently exercises the hexagonal inherent-vs-trait `distance` duplication directly (e.g. asserting both return the same numeric value for the same input pair).
