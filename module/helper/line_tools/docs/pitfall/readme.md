# Pitfall Doc Definition

A **pitfall** documents one way this crate's API can be misused or misunderstood — the trap, why it happens, and how to avoid it. In `line_tools`, these are confirmed edge cases in the geometry and shader math — point spacing, join angles, and degenerate input — that the shaders alone won't reveal, each entry recording what goes wrong and how, or whether, it's addressed. This collection holds one instance per known pitfall; the table below is the index into them.

### Scope

- **Purpose**: `line_tools`' geometry and shader math contain confirmed edge cases in point spacing, join angles, and degenerate input that are not obvious from reading the shaders alone.
- **Responsibility**: Document each confirmed trap, its observable failure, and its mitigation (or lack thereof).
- **In Scope**: Confirmed rendering-geometry pitfalls in `line_tools`' 2D and 3D line implementations.
- **Out of Scope**: General WebGL/GLSL pitfalls not specific to `line_tools`' design.

### Overview Table

| ID | Name | Purpose | Status |
|----|------|---------|--------|
| 001 | [Overlapping Geometry at Joins and Caps](001_overlapping_geometry_at_joins_and_caps.md) | Blending exposes seams between separately-drawn join/cap/body geometry | ✅ |
| 002 | [Small-Angle Segment Overlap](002_small_angle_segment_overlap.md) | Miter join overlap clamp for small angles | ✅ |
| 003 | [Close-Points Segment Overlap](003_close_points_segment_overlap.md) | Segments overlap heavily when points are closer than the line width | ⚠️ |
| 004 | [Zero-Length Segment Break](004_zero_length_segment_break.md) | Coincident points break the line; guarded in 2D only | ⚠️ |
| 005 | [Non-Neighbor Segment Overlap](005_non_neighbor_segment_overlap.md) | Side effect of the small-angle clamp | ⚠️ |
| 006 | [Parallel-Segment Division by Zero](006_parallel_segment_division_by_zero.md) | Degenerate tangent when neighbouring segments are parallel | ✅ |
| 007 | [UV Coordinate Flip at Width](007_uv_coordinate_flip_at_width.md) | UV coordinates shrink and flip sign as width grows | ✅ |
