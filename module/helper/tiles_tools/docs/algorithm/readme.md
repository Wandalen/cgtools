# Algorithm Doc Definition

An **algorithm** instance documents one nontrivial computation this crate performs, worked through precisely enough to reimplement or audit. In `tiles_tools`, that means each of the crate's deterministic, coordinate-generic procedures — distance and neighbor formulas, A* pathfinding, field-of-view calculation, hexagon mesh generation, coordinate system conversion — with any divergence between an implementation and what its name or a sibling comment claims called out explicitly. This collection holds one instance per algorithm; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `tiles_tools`' deterministic, coordinate-generic procedures.
- **Responsibility**: Document each algorithm's step-by-step computation, and disclose where an implementation diverges from what its name or a sibling comment claims.
- **In Scope**: Distance/neighbor formulas, A* pathfinding, field-of-view calculation, hexagon mesh generation, coordinate system conversion.
- **Out of Scope**: The types these algorithms operate on (see `type/`), the runtime API that triggers them (see `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Coordinate Distance & Neighbor Formulas](001_coordinate_distance_and_neighbor_formulas.md) | Per-system `Distance`/`Neighbors` formulas; hexagonal's dual `distance` methods | ✅ |
| 002 | [Generic A* Pathfinding](002_generic_astar_pathfinding.md) | `astar`/`astar_with_edge_costs`/`astar_advanced`; `allow_diagonal`'s unused status | ✅ |
| 003 | [Field of View Calculation](003_field_of_view_calculation.md) | `FOVAlgorithm`'s 4 variants; visibility/lighting types | ✅ |
| 004 | [Hexagon Geometry Generation](004_hexagon_geometry_generation.md) | Independent-primitive vertex buffers (`TRIANGLES`/`LINES`, never fan/loop modes) | ✅ |
| 005 | [Coordinate System Conversion](005_coordinate_system_conversion.md) | Exact vs. approximate conversion traits; `triangular`'s zero conversion paths | ✅ |
