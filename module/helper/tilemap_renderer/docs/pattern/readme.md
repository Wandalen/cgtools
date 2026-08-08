# Pattern Doc Definition

### Scope

- **Purpose**: `tilemap_renderer`'s backend-agnostic design rests on one architectural pattern that every adapter implements.
- **Responsibility**: Document that pattern's problem, solution shape, applicability, and trade-offs.
- **In Scope**: The crate-wide core/adapter architectural pattern.
- **Out of Scope**: Per-adapter implementation status (see `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Ports and Adapters Backend Architecture](001_ports_and_adapters_backend_architecture.md) | Core/adapter split behind one `Backend` trait, feature-gated in a single crate | ✅ |
