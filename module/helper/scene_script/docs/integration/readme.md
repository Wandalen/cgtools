# Integration Doc Definition

An **integration** documents a system boundary crossed at runtime — how an external system is accessed, how its errors are handled, and what compatibility it requires. In `scene_script`, this collection is the navigational hub for the one external-system boundary this crate crosses: the embedded Rhai interpreter. This collection holds one instance per external-system boundary; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `scene_script`'s external-system runtime boundaries.
- **Responsibility**: Document how the boundary is crossed, how failures on it are handled, and what compatibility it requires.
- **In Scope**: The Rhai `Engine`/`AST`/`Dynamic` embedding boundary.
- **Out of Scope**: Why `rhai` was selected and its feature configuration (see `dependency/001`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Rhai Engine Boundary](001_rhai_engine_boundary.md) | How values, calls, and errors cross the Rust ↔ Rhai runtime boundary | ✅ |
