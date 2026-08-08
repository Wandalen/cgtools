# API Doc Definition

### Scope

- **Purpose**: Navigational hub for `tiles_tools`' public runtime operation surfaces.
- **Responsibility**: Document each API's operations, error handling, and compatibility guarantees.
- **In Scope**: The ECS `World` runtime.
- **Out of Scope**: Save-file persistence operations (see `persistence/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [ECS World Runtime API](001_ecs_world_runtime_api.md) | `World`'s full operation set; direct `hecs` type exposure; the one no-op operation | ✅ |
