# Data Structure Doc Definition

A **data structure** instance documents one storage or indexing structure the crate uses internally — its layout and complexity characteristics. In `tiles_tools`, that means the crate's core storage containers — `Grid2D` dense storage and the `Quadtree` spatial index — with each one's structure, operations, and invariants written down. This collection holds one instance per structure; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `tiles_tools`' core storage containers.
- **Responsibility**: Document each container's structure, operations, and invariants.
- **In Scope**: `Grid2D` dense storage, `Quadtree` spatial index.
- **Out of Scope**: The coordinate types these containers are indexed by (see `type/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Grid2D Dense Hex-Bounded Storage](001_grid2d_dense_hex_bounded_storage.md) | Coordinate-indexed dense array storage | ✅ |
| 002 | [Spatial Quadtree](002_spatial_quadtree.md) | Recursive spatial subdivision for range queries | ✅ |
