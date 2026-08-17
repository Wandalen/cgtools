# Invariant: Triangular Coordinate Sum Constraint

### Scope

- **Purpose**: State the property that a triangular coordinate's three components must sum to `1` or `2`, and document precisely where that property is checked versus assumed.
- **Responsibility**: Document the invariant statement, its one true enforcement boundary (`Coordinate::new`), and the internal fast path (`new_unchecked`) that bypasses the check while relying on provably sum-preserving arithmetic.
- **In Scope**: `triangular::Coordinate<Orientation>`'s `a + b + c ∈ {1, 2}` constraint; `new` vs. `new_unchecked`; `Deserialize`'s use of the checked path; `neighbors()`'s use of the unchecked path.
- **Out of Scope**: The dual/conversion algorithms that consume valid triangular coordinates once constructed (see `algorithm/001`, `algorithm/005`).

### Invariant Statement

For every `triangular::Coordinate<Orientation> { a, b, c, .. }`, the sum `a + b + c` MUST equal exactly `1` or `2`. A sum of `2` identifies an "up/right" triangle (`is_up_or_right()`); a sum of `1` identifies a "down/left" triangle (`is_down_or_left()`) (`src/coordinates/triangular.rs:159-162`). No third sum value is ever valid — the two triangle orientations that tile a triangular grid are exhaustively distinguished by this one integer.

### Enforcement Mechanism

**The one checked public boundary** is `Coordinate::new(a, b, c) -> Option<Self>` (`src/coordinates/triangular.rs:121-131`): it computes the sum as `i64` (avoiding `i32` overflow on the addition itself) and returns `Some` only if `(1..=2).contains(&sum)`, `None` otherwise. This is the sole public, checked constructor — any external caller building a triangular coordinate from arbitrary `a`/`b`/`c` values goes through this `Option`-returning path and cannot construct an invalid one.

**The internal fast path** is `pub(crate) const fn new_unchecked(a, b, c) -> Self` (`src/coordinates/triangular.rs:137-140`) — crate-internal only (not reachable from outside `tiles_tools`), performing no sum check at all. It is used in exactly the contexts verified below, each of which is provably sum-preserving by construction rather than by runtime check:

- **`Deserialize`** (`src/coordinates/triangular.rs:58`): the hand-written `impl<'de, Orientation> Deserialize<'de>` first deserializes into a plain `{a, b, c}` helper, computes the sum, and returns a serde error via the same `1..=2` check `new` performs (`src/coordinates/triangular.rs:53`) *before* calling `new_unchecked` to build the final value (line 58) — the check still happens, just inline in the `Deserialize` impl rather than by delegating to `new`.
- **`Clone`** (`src/coordinates/triangular.rs:88`): clones an already-valid coordinate's fields verbatim — cannot change the sum.
- **`neighbors()`** (`src/coordinates/triangular.rs:191-201`): computes `offset = -is_up_or_right + is_down_or_left`, which is `-1` when the source triangle sums to `2` (shifting exactly one of `a`/`b`/`c` by `-1` yields sum `1`) and `+1` when the source sums to `1` (shifting one component by `+1` yields sum `2`) — every one of the three emitted neighbors has a sum in `{1, 2}` by construction, verified directly from the offset arithmetic, not merely assumed.
- **`from_pixel_with_edge_len`** (two `impl` blocks, one per `Orientation`, `src/coordinates/triangular.rs:224-273`): derives `a`/`b`/`c` from a geometric formula over the input pixel position; not algebraically re-verified here as sum-preserving, but is the one `new_unchecked` call site whose safety rests on a geometric argument rather than a simple integer-offset argument.

### Violation Consequences

Because the only reachable *external* constructor (`new`) is checked, and every *internal* `new_unchecked` call site verified above is either preceded by an explicit check (`Deserialize`) or is provably sum-preserving by its own arithmetic (`neighbors()`), no currently-shipped code path can produce a `triangular::Coordinate` violating the invariant. The residual risk is prospective, not present: `new_unchecked` is `pub(crate)`, not `private` — any *future* internal contributor adding a new derivation (a new `From` impl, a new geometric helper) can call it directly without the compiler enforcing the sum constraint, and would need to re-derive (or at least re-check) sum-preservation by hand, the same way `neighbors()`'s implementer evidently did. There is no compile-time guard against a future unchecked call site that isn't actually sum-preserving.

### Algorithms

| File | Relationship |
|------|--------------|
| [algorithm/001_coordinate_distance_and_neighbor_formulas.md](../algorithm/001_coordinate_distance_and_neighbor_formulas.md) | `neighbors()`'s sum-preserving offset derivation, verified above, is this invariant's clearest evidence of correctness |

### Data Structures

| File | Relationship |
|------|--------------|
| [data_structure/001_grid2d_dense_hex_bounded_storage.md](../data_structure/001_grid2d_dense_hex_bounded_storage.md) | Not applicable — `Grid2D` is hexagonal-only; a triangular-coordinate grid needing this invariant would require its own storage type |
| [data_structure/002_spatial_quadtree.md](../data_structure/002_spatial_quadtree.md) | Not applicable — `triangular::Coordinate` does not implement `SpatialCoordinate` and cannot be used with `Quadtree` directly |

### Types

| File | Relationship |
|------|--------------|
| [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) | `triangular::Coordinate<Orientation>` is the one type in that doc's table carrying a field-value invariant beyond phantom-type grid separation |

### Sources

| File | Relationship |
|------|--------------|
| `src/coordinates/triangular.rs` | `Coordinate::new`, `Coordinate::new_unchecked`, `Deserialize` impl, `Neighbors` impl, `is_up_or_right`/`is_down_or_left` |

### Tests

No dedicated regression test currently pins `new_unchecked`'s `pub(crate)` visibility as intentional, or exercises a hand-constructed invalid sum via any of the verified internal call sites to confirm they in fact reject it — the sum-preservation argument above is a source-code proof, not an executed assertion.
