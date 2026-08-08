# Algorithm Doc Definition

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
| 004 | [Hexagon Mesh Generation](004_hexagon_mesh_generation.md) | Fan-triangulated vertex buffers; the `tranform` misspelling | ✅ |
| 005 | [Coordinate System Conversion](005_coordinate_system_conversion.md) | Exact vs. approximate conversion traits; `triangular`'s zero conversion paths | ✅ |
