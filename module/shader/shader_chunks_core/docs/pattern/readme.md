# Pattern Doc Definition

A **pattern** here is a reusable consumer-side form — how an application is
meant to hold and combine chunks — distinct from the cross-crate design
rules in the repo-root `docs/pattern/`. Two exist, and they are the two
sources a composed set draws rows from: importing bundled chunks by name,
and defining app-local chunks beside them. This collection holds one
instance per form; the table below is the index into them.

### Scope

- **Purpose**: Document the two forms consumer code takes when building a shader from chunks, so every consumer wires them the same way.
- **Responsibility**: Describe each form's problem, solution, applicability bounds, trade-offs, and a worked example.
- **In Scope**: Selective const import of bundled chunks; crate-local chunk definition and its mixing with imports.
- **Out of Scope**: The composition procedure both feed (see `algorithm/`); the guarantees that keep the forms honest (see `invariant/`); ecosystem-wide patterns (see repo-root `docs/pattern/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Selective Const Import](001_selective_const_import.md) | Import only the bundled chunks an application names, at compile time — a typo fails the build | ✅ |
| 002 | [Crate-Local Chunk](002_crate_local_chunk.md) | Define app-specific chunks as descriptor literals that mix freely with imported rows | ✅ |
