# API Doc Definition

An **api** instance documents one integration surface — the entry points external code uses to drive this crate. In `tiles_tools`, the one surface in scope today is the ECS `World` runtime, with its operations, error handling, and compatibility guarantees written down so external callers can drive it without reading the implementation. This collection holds one instance per API surface; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `tiles_tools`' public runtime operation surfaces.
- **Responsibility**: Document each API's operations, error handling, and compatibility guarantees.
- **In Scope**: The ECS `World` runtime.
- **Out of Scope**: Save-file persistence operations (see `persistence/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [ECS World Runtime API](001_ecs_world_runtime_api.md) | `World`'s full operation set; direct `hecs` type exposure; the one no-op operation | ✅ |
