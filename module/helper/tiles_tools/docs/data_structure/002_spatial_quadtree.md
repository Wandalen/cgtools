# Data Structure: Spatial Quadtree

### Scope

- **Purpose**: Document `Quadtree<C>`, the sparse spatial-partitioning structure used for range/circle queries over dynamically-positioned entities.
- **Responsibility**: Document its node structure, the `SpatialCoordinate` trait boundary that limits which coordinate types can use it, and the source-verified gap between `remove`'s doc comment and its actual algorithmic complexity.
- **In Scope**: `SpatialBounds`, `SpatialEntity<C>`, `SpatialCoordinate`, `Quadtree<C>`'s node structure, `insert`/`remove`/`query_region`/`query_circle`.
- **Out of Scope**: The dense hex-bounded grid used for per-cell (rather than per-entity) storage (see `data_structure/001`); field-of-view/range algorithms that consume quadtree query results (see `algorithm/003`).

### Abstract

`Quadtree<C>` is a recursive four-way spatial partition (`src/spatial.rs`'s own module doc comment advertises *"Fast Collision Detection: O(log n) instead of O(n²) for entity pairs"*, `src/spatial.rs:10`) storing `SpatialEntity<C>` values keyed by an `(x, y)` bounding region rather than by grid coordinate. Unlike `Grid2D` (`data_structure/001`), which is dense and bound to one specific coordinate system, `Quadtree<C>` is generic over any `C: SpatialCoordinate` — but that trait is implemented for only two types in the entire crate, not for every coordinate system `type/001` documents.

### Structure

```
pub struct Quadtree< C >
{
  root    : QuadtreeNode< C >,
  bounds  : SpatialBounds,
  max_entities : usize,   // split threshold per leaf
  max_depth   : usize,    // (tracked; see Operations)
}

enum QuadtreeNode< C >
{
  Leaf     { entities : Vec< SpatialEntity< C > > },
  Internal { northeast: Box<Self>, northwest: Box<Self>, southeast: Box<Self>, southwest: Box<Self> },
}
```

`SpatialEntity<C>` pairs an `id: u32` with a `C` position; `SpatialBounds` is an axis-aligned rectangle used both for the tree's own quadrant subdivision and as the query-region argument shape.

**`SpatialCoordinate` is implemented for exactly two types**: the plain tuple `(i32, i32)` and `square::Coordinate<T>` (`src/spatial.rs:517`, `527` — generic over `square`'s own `Connectivity` parameter). None of `hexagonal::Coordinate`, `triangular::Coordinate`, `isometric::Coordinate`, or `Pixel` implement it (see `type/001`'s full type list). A hex-grid or triangular-grid game wanting quadtree-backed spatial queries must convert its coordinates to `(i32, i32)` itself before constructing a `Quadtree<(i32, i32)>` — there is no `Quadtree<hexagonal::Coordinate<...>>` usable directly.

### Operations

| Operation | Signature (conceptual) | Complexity (verified) |
|-----------|--------------------------|------------------------|
| `insert` | `(&mut self, SpatialEntity<C>)` | Recurses through `insert_recursive_static`, splitting a `Leaf` into an `Internal` node once `max_entities` is exceeded — genuine O(log n) descent, one path from root to the target leaf. |
| `query_region` | `(&self, &SpatialBounds) -> Vec<SpatialEntity<C>>` | Recurses only into child quadrants whose bounds intersect the query region — genuine pruned traversal, not a full scan. |
| `query_circle` | `(&self, center_x, center_y, radius) -> Vec<SpatialEntity<C>>` where `C: Distance` | Implemented as `query_region` over the circle's bounding square, then a `.filter()` by exact `Distance` to discard corner false-positives (`src/spatial.rs`) — correct, but pays a rectangular query's full cost even for a circle much smaller than its bounding box. |
| `remove` | `(&mut self, entity_id: u32) -> Vec<SpatialEntity<C>>` | **Divergence, verified directly against source**: `remove`'s doc comment (*"Removes all entities with the specified ID from the quadtree"*) makes no complexity claim itself, but the module-level *"O(log n)"* banner (`src/spatial.rs:10`) describes the structure generally, and `remove` is the operation that does not meet it. `remove_recursive_static`'s `Internal`-node arm unconditionally recurses into **all four children** — `northeast`, `northwest`, `southeast`, `southwest`, every call, no bounds pruning — because the function receives only an opaque `entity_id : u32`, with no positional hint to prune the search by. This is a full O(n) tree walk, structurally unable to be O(log n) as written: unlike `query_region`/`query_circle`, `remove` has no `SpatialBounds` argument to intersect against. |
| `all_entities` / `clear` | `(&self) -> Vec<SpatialEntity<C>>` / `(&mut self)` | Full-tree collection / reset; O(n) is expected and accurate for both. |

### Invariants

| File | Relationship |
|------|--------------|
| [invariant/001_triangular_coordinate_sum_constraint.md](../invariant/001_triangular_coordinate_sum_constraint.md) | Not applicable — `triangular::Coordinate` does not implement `SpatialCoordinate` and cannot be used with `Quadtree` directly |

### Types

| File | Relationship |
|------|--------------|
| [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) | `SpatialCoordinate`'s two implementors (`(i32, i32)`, `square::Coordinate<T>`) are a small subset of that doc's full coordinate-type table |

### Sources

| File | Relationship |
|------|--------------|
| `src/spatial.rs` | `SpatialBounds`, `SpatialEntity<C>`, `SpatialCoordinate`, `Quadtree<C>`, `QuadtreeNode<C>`, `QuadtreeStats` |

### Tests

No dedicated regression test currently pins `remove`'s actual O(n) cost against the module doc comment's general O(log n) claim — both are real (verified against `src/spatial.rs`), but no benchmark or assertion distinguishes `remove`'s complexity from `insert`/`query_region`'s.
