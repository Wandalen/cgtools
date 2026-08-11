# Pattern Doc Definition

A **pattern** here is a reusable design rule this crate itself is built on — distinct from the ecosystem-wide patterns in the workspace root's `docs/pattern/`, this one is scoped to this crate's own internal design. In `tilemap_renderer`, that means the one architectural pattern every backend adapter implements to stay backend-agnostic, documented here with its problem, solution shape, applicability, and trade-offs. This collection holds one instance per pattern; the table below is the index into them.

### Scope

- **Purpose**: `tilemap_renderer`'s backend-agnostic design rests on one architectural pattern that every adapter implements.
- **Responsibility**: Document that pattern's problem, solution shape, applicability, and trade-offs.
- **In Scope**: The crate-wide core/adapter architectural pattern.
- **Out of Scope**: Per-adapter implementation status (see `feature/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Ports and Adapters Backend Architecture](001_ports_and_adapters_backend_architecture.md) | Core/adapter split behind one `Backend` trait, feature-gated in a single crate | ✅ |
