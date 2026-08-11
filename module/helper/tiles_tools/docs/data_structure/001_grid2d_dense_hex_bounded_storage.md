# Data Structure: Grid2D Dense Hex-Bounded Storage

### Scope

- **Purpose**: Document `Grid2D<System, Orientation, T>`, the dense rectangular-array-backed store for per-cell values.
- **Responsibility**: Document its actual (hexagonal-only) coordinate binding despite generic-looking type parameters, its construction/indexing operations, and the two source-verified divergences between its doc comments and its actual behavior.
- **In Scope**: Internal layout (`ndarray`-backed `Array2<T>`), construction, `get`/`get_mut`/`insert`/`remove`/`Index`/`IndexMut` operations, the `new_uncheked` naming typo.
- **Out of Scope**: The coordinate types it is indexed by (see `type/001`); the quadtree used for sparse/dynamic spatial storage instead of this dense grid (see `data_structure/002`).

### Abstract

`Grid2D<System, Orientation, T>` is a dense, rectangular-bounded store mapping every coordinate in a fixed min/max range to a value of type `T`, backed directly by an `ndarray::Array2<T>`. Despite its name and its two leading type parameters, it is **not coordinate-system-generic** — `src/collection.rs`'s own module doc comment states it plainly: *"designed to store data mapped to hexagonal coordinates... generic over the hexagonal coordinate system and orientation"* (`src/collection.rs:1-3`), and the struct's only coordinate import is `use crate::coordinates::hexagonal::Coordinate;` (`src/collection.rs:5`) — not a generic `Coordinate` trait, the concrete hexagonal type. `System`/`Orientation` select *which* hexagonal marker combination (`Axial`/`Offset<Parity>` × `Pointy`/`Flat` — see `type/001`) the grid is keyed by; they do not make the grid usable with `square::Coordinate`, `triangular::Coordinate`, or `isometric::Coordinate`. A `Grid2D` for a square or triangular game board is not constructible from this type as written.

### Structure

```
pub struct Grid2D< System, Orientation, T >
{
  data : Array2< T >,                                        // dense backing store
  min  : I64x2,                                               // grid-space origin offset
  _marker : PhantomData< Coordinate< System, Orientation > >,  // binds System/Orientation, hexagonal only
}
```

`min` holds the coordinate-space offset of the array's `(0, 0)` cell, so the grid can represent any contiguous hexagonal-coordinate rectangle, not just one anchored at the origin. Indexing translates a `Coordinate<System, Orientation>` to an `(i, j)` array index via `(coord.r - min[1], coord.q - min[0])` (`src/collection.rs:163-166`).

### Operations

| Operation | Signature (conceptual) | Behavior |
|-----------|--------------------------|----------|
| `with_size_and_fn` | `(min, max, F) -> Self` where `F: Fn(Coordinate) -> T` | Constructs a grid over the given coordinate range, filling each cell via the supplied function. Doc comment: *"Panics if the size derived from the min/max coordinates is negative or cannot be converted to `usize`"* (`src/collection.rs:26-27`) — this constructor's panic behavior is accurate, unlike `get`/`get_mut` below. |
| `with_size_and_default` | `(min, max) -> Self` where `T: Default` | Same range semantics, cells filled with `T::default()`. |
| `iter` / `iter_mut` | `(&self) -> Iter<T>` / `(&mut self) -> IterMut<T>` | Raw value iteration, no coordinate attached. |
| `indexed_iter` | `(&self) -> impl Iterator<Item = (Coordinate<System, Orientation>, &T)>` | Pairs each value with its coordinate, reconstructed via `Coordinate::new_uncheked(j, i)` (`src/collection.rs:76`) — note the misspelling: `new_uncheked`, missing the middle `h` in "unchecked." The same misspelled name is used consistently across the crate (also `hexagonal::Coordinate<Offset<Parity>, _>::new`, `src/coordinates/hexagonal.rs:143`), so it is the actual, load-bearing internal API spelling, not an isolated one-off. |
| `get` | `(&self, coord: C) -> Option<&T>` where `C: Into<Coordinate<System, Orientation>>` | See Divergence below — doc comment claims panic behavior the body does not have. |
| `get_mut` | `(&mut self, coord: C) -> Option<&mut T>` | Same divergence as `get`. |
| `insert` / `remove` | `(&mut self, coord: C, value: T) -> Option<T>` / `(&mut self, coord: C) -> Option<T>` | Only implemented for `Grid2D<System, Orientation, Option<T>>` (`impl` block at `src/collection.rs:116`) — a grid must be instantiated with an `Option`-wrapped value type to support sparse insert/remove semantics; a `Grid2D<_, _, T>` for a non-`Option` `T` has no `insert`/`remove`. |
| `Index` / `IndexMut` | `(&self, coord: C) -> &T` / `(&mut self, coord: C) -> &mut T` | Unlike `get`/`get_mut`, these standard-trait accessors panic on an out-of-range coordinate — this is the operation pair whose panic behavior actually matches what `get`/`get_mut`'s doc comments (incorrectly) describe. |

**Divergence, verified directly against source**: `get` and `get_mut` both carry the doc comment *"# Panics / Panics if the coordinate is out of the grid's bounds"* (`src/collection.rs:155-157`, `171-173`), but neither body can panic on an out-of-bounds coordinate. `get`'s body (`src/collection.rs:162-167`) converts the coordinate offset via `.try_into().ok()?` — an out-of-range subtraction short-circuits to `None` through the `?` operator — and then calls `self.data.get(...)`, `ndarray`'s own checked accessor, which itself returns `None` rather than panicking on an out-of-range index. `get_mut` (`src/collection.rs:178-183`) follows the identical pattern. Both functions return `Option` precisely *because* they handle the out-of-bounds case by returning `None`, not by panicking — the doc comment describes the behavior of `Index`/`IndexMut` (the panicking pair above), apparently copy-pasted without updating for the `Option`-returning pair it was attached to.

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_triangular_coordinate_sum_constraint.md](../invariant/001_triangular_coordinate_sum_constraint.md) | Not applicable to `Grid2D` itself (hexagonal-only) — cross-referenced to make the boundary explicit: a triangular-coordinate grid would need its own storage type, not this one |

### Types

| File | Relationship |
|------|--------------|
| [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) | `System`/`Orientation` select among the hexagonal marker combinations documented there; `Grid2D` cannot be parameterized with any non-hexagonal type from that doc |

### Sources

| File | Relationship |
|------|--------------|
| `src/collection.rs` | `Grid2D`, all operations in the table above |

### Tests

No dedicated regression test currently pins `get`/`get_mut`'s actual (non-panicking) out-of-bounds behavior against their doc comments' (panicking) claim — the divergence is real in the shipped source but not exercised by an assertion either way.
