# Feature Doc Definition

A **feature** is a navigational hub collecting every source file, test file, and doc instance for one self-contained, user-facing capability, without restating their content. In `scene_script`, this collection has exactly one instance: the crate's sole capability, scripting a 2D scene or its animation in Rhai. The table below is the index into it.

### Scope

- **Purpose**: Navigational hub for `scene_script`'s end-to-end scripting capability.
- **Responsibility**: Point to every artifact (source, test, and doc instance) implementing this capability.
- **In Scope**: The whole of `scene_script`'s public surface and design documentation.
- **Out of Scope**: Restating content already owned by another collection — this hub cross-references, never duplicates.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Rhai Scene Scripting](001_rhai_scene_scripting.md) | The crate's whole capability at a glance, including its current scope boundary | ✅ |
