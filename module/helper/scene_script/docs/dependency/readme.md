# Dependency Doc Definition

A **dependency** documents an external crate's selection rationale, rejected alternatives, and configuration. In `scene_script`, this collection is the navigational hub for the one dependency whose configuration carries non-obvious risk: `rhai`'s `internals` feature. This collection holds one instance per documented dependency; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `scene_script`'s externally-selected crates whose configuration needs a durable rationale.
- **Responsibility**: Document why each crate was chosen, alternatives considered, and known issues.
- **In Scope**: The `rhai` dependency and its `internals` feature flag.
- **Out of Scope**: Workspace-internal sibling crates (`ndarray_cg`, `animation`, `mod_interface`) — not external selections requiring alternatives-rejected framing; the operational boundary of using `rhai` at runtime (see `integration/`).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [rhai (internals feature)](001_rhai_internals_feature.md) | Why the `internals` feature is enabled and the upgrade risk it carries | ✅ |
