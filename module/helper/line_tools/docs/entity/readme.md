# Doc Entities

## Master Doc Entities Table

| Type | Purpose | Master File | Instances |
|------|---------|-------------|----------:|
| `feature/` | End-to-end 2D/3D line rendering capabilities, cross-referencing source and pitfalls | [feature/readme.md](feature/readme.md) | 2 |
| `pitfall/` | Confirmed rendering-geometry traps, their failure modes, and mitigations | [pitfall/readme.md](pitfall/readme.md) | 7 |

## Master Doc Instances Table

| Entity  | ID  | Name                                    | File                                                                                                            |
|---------|-----|------------------------------------------|-------------------------------------------------------------------------------------------------------------------|
| feature | 001 | 2D Line Rendering                        | [feature/001_2d_line_rendering.md](feature/001_2d_line_rendering.md)                                             |
| feature | 002 | 3D Line Rendering                        | [feature/002_3d_line_rendering.md](feature/002_3d_line_rendering.md)                                             |
| pitfall | 001 | Overlapping Geometry at Joins and Caps   | [pitfall/001_overlapping_geometry_at_joins_and_caps.md](pitfall/001_overlapping_geometry_at_joins_and_caps.md)   |
| pitfall | 002 | Small-Angle Segment Overlap              | [pitfall/002_small_angle_segment_overlap.md](pitfall/002_small_angle_segment_overlap.md)                         |
| pitfall | 003 | Close-Points Segment Overlap             | [pitfall/003_close_points_segment_overlap.md](pitfall/003_close_points_segment_overlap.md)                       |
| pitfall | 004 | Zero-Length Segment Break                | [pitfall/004_zero_length_segment_break.md](pitfall/004_zero_length_segment_break.md)                             |
| pitfall | 005 | Non-Neighbor Segment Overlap             | [pitfall/005_non_neighbor_segment_overlap.md](pitfall/005_non_neighbor_segment_overlap.md)                       |
| pitfall | 006 | Parallel-Segment Division by Zero        | [pitfall/006_parallel_segment_division_by_zero.md](pitfall/006_parallel_segment_division_by_zero.md)             |
| pitfall | 007 | UV Coordinate Flip at Width              | [pitfall/007_uv_coordinate_flip_at_width.md](pitfall/007_uv_coordinate_flip_at_width.md)                         |
