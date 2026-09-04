# Type Doc Definition

A **type** instance documents one significant data type's shape and role in the crate. In `tiles_tools`, that covers the generic `Coordinate<System, Orientation>` model spanning all four grid topologies and the ECS component vocabulary, each written down with its structural role, phantom-type parameters, and validation rules. This collection holds one instance per type; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `tiles_tools`' core generic type contracts.
- **Responsibility**: Document each type's structural role, phantom-type parameters, and validation rules.
- **In Scope**: The generic `Coordinate<System, Orientation>` model spanning all 4 grid topologies; the ECS component vocabulary.
- **Out of Scope**: Concrete algorithms operating on these types (see `algorithm/`), runtime API operations (see `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Coordinate System Type Model](001_coordinate_system_type_model.md) | The `Coordinate<System, Orientation>` phantom-type pattern across hexagonal/square/triangular/isometric | ✅ |
| 002 | [ECS Component Vocabulary](002_ecs_component_vocabulary.md) | The 13 components spawned/queried through `ecs::World` | ✅ |
