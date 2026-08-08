# Data Structure Doc Definition

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
