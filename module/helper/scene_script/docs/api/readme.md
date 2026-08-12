# API Doc Definition

An **api** documents a public programmatic interface exposed to external callers. In `scene_script`, this collection is the navigational hub for the Rhai-facing scripting surface — everything a script can actually call once `engine_build()` has run. This collection holds one instance per distinct interface; the table below is the index into them.

### Scope

- **Purpose**: Navigational hub for `scene_script`'s script-callable operations surface.
- **Responsibility**: Document available operations, their error conditions, and compatibility guarantees.
- **In Scope**: Constructors, methods, operators, and property getters registered into the `rhai::Engine`.
- **Out of Scope**: The shape/fields of the registered types themselves (see `data_structure/`); the Rust-level registration functions' own rustdoc (see crate [`readme.md`](../../readme.md)'s Responsibility Table).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Rhai Scripting Surface](001_rhai_scripting_surface.md) | Every constructor, method, operator, and getter a script can call | ✅ |
