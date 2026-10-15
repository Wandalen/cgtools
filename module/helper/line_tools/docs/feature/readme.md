# Feature Doc Definition

A **feature** instance documents one cohesive slice of the crate's public API. In `line_tools`, each instance covers one configurable, anti-aliased 2D or 3D polyline rendering capability, linking out to the source, shaders, and pitfalls behind it. This collection holds one instance per feature; the table below is the index into them.

### Scope

- **Purpose**: `line_tools`' user-facing rendering capabilities exist to give 2D and 3D applications configurable, anti-aliased polyline rendering without hand-writing WebGL.
- **Responsibility**: Document each end-to-end rendering feature as a navigational hub over its source, shaders, and known pitfalls.
- **In Scope**: 2D and 3D line rendering capabilities exposed by `line_tools`' public API.
- **Out of Scope**: Implementation-level geometry/shader detail (see the Sources/Pitfalls references inside each instance).

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [2D Line Rendering](001_2d_line_rendering.md) | Caps, joins, and incremental point editing for 2D polylines | ✅ |
| 002 | [3D Line Rendering](002_3d_line_rendering.md) | Camera-facing billboarded 3D polylines with width modes, colors, dashing, and anti-aliasing | ✅ |
