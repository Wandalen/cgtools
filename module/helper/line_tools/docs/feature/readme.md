# Feature Doc Entity

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
