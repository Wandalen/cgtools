# Doc Definitions

## Master Doc Definitions Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `algorithm/` | Deterministic, coordinate-generic procedures: distance/neighbor formulas, A* pathfinding, field of view, hex mesh generation, coordinate conversion | [algorithm/readme.md](../algorithm/readme.md) | 5 |
| `api/` | Public runtime operation surfaces: the ECS `World` | [api/readme.md](../api/readme.md) | 1 |
| `architectural_evaluation/` | Structured multi-alternative architectural decisions: ECS library selection | [architectural_evaluation/readme.md](../architectural_evaluation/readme.md) | 1 |
| `data_structure/` | Core storage containers: `Grid2D`, `Quadtree` | [data_structure/readme.md](../data_structure/readme.md) | 2 |
| `invariant/` | Correctness properties that must always hold: triangular coordinate sum constraint, lattice address primacy | [invariant/readme.md](../invariant/readme.md) | 2 |
| `persistence/` | On-disk save-file formats: storage model, data layout, durability | [persistence/readme.md](../persistence/readme.md) | 1 |
| `pitfall/` | Known traps in the current implementation, their failure modes, and mitigations | [pitfall/readme.md](../pitfall/readme.md) | 3 |
| `type/` | Core generic type contracts: coordinate system model, ECS component vocabulary | [type/readme.md](../type/readme.md) | 2 |

## Master Doc Instances Table

| Definition | ID | Name | File |
|--------|-----|------|------|
| algorithm | 001 | Coordinate Distance & Neighbor Formulas | [algorithm/001_coordinate_distance_and_neighbor_formulas.md](../algorithm/001_coordinate_distance_and_neighbor_formulas.md) |
| algorithm | 002 | Generic A* Pathfinding | [algorithm/002_generic_astar_pathfinding.md](../algorithm/002_generic_astar_pathfinding.md) |
| algorithm | 003 | Field of View Calculation | [algorithm/003_field_of_view_calculation.md](../algorithm/003_field_of_view_calculation.md) |
| algorithm | 004 | Hexagon Geometry Generation | [algorithm/004_hexagon_geometry_generation.md](../algorithm/004_hexagon_geometry_generation.md) |
| algorithm | 005 | Coordinate System Conversion | [algorithm/005_coordinate_system_conversion.md](../algorithm/005_coordinate_system_conversion.md) |
| api | 001 | ECS World Runtime API | [api/001_ecs_world_runtime_api.md](../api/001_ecs_world_runtime_api.md) |
| architectural_evaluation | 001 | ECS Library Selection | [architectural_evaluation/001_ecs_library_selection.md](../architectural_evaluation/001_ecs_library_selection.md) |
| data_structure | 001 | Grid2D Dense Hex-Bounded Storage | [data_structure/001_grid2d_dense_hex_bounded_storage.md](../data_structure/001_grid2d_dense_hex_bounded_storage.md) |
| data_structure | 002 | Spatial Quadtree | [data_structure/002_spatial_quadtree.md](../data_structure/002_spatial_quadtree.md) |
| invariant | 001 | Triangular Coordinate Sum Constraint | [invariant/001_triangular_coordinate_sum_constraint.md](../invariant/001_triangular_coordinate_sum_constraint.md) |
| invariant | 002 | Lattice Address Primacy | [invariant/002_lattice_address_primacy.md](../invariant/002_lattice_address_primacy.md) |
| persistence | 001 | Save File Model | [persistence/001_save_file_model.md](../persistence/001_save_file_model.md) |
| pitfall | 001 | Flow Field Algorithm Unimplemented | [pitfall/001_flow_field_algorithm_unimplemented.md](../pitfall/001_flow_field_algorithm_unimplemented.md) |
| pitfall | 003 | Save-File Compression Is a Fake Wrapper | [pitfall/003_savefile_compression_is_a_fake_wrapper.md](../pitfall/003_savefile_compression_is_a_fake_wrapper.md) |
| pitfall | 004 | Hexagonal Axial Distance Method Ambiguity | [pitfall/004_hexagonal_axial_distance_method_ambiguity.md](../pitfall/004_hexagonal_axial_distance_method_ambiguity.md) |
| type | 001 | Coordinate System Type Model | [type/001_coordinate_system_type_model.md](../type/001_coordinate_system_type_model.md) |
| type | 002 | ECS Component Vocabulary | [type/002_ecs_component_vocabulary.md](../type/002_ecs_component_vocabulary.md) |
