# Architectural Evaluation Doc Entity

### Scope

- **Purpose**: Navigational hub for structured multi-alternative architectural decisions in `tiles_tools`.
- **Responsibility**: Document each evaluation's concern legend, trade-off matrix, and verdict.
- **In Scope**: ECS library selection.
- **Out of Scope**: The shipped API resulting from each decision (see `api/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [ECS Library Selection](001_ecs_library_selection.md) | `hecs` vs `bevy_ecs` vs `specs`, migrated from `docs/ecs_decision.md` | ✅ |
