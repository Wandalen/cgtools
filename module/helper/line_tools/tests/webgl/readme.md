### Responsibility Table

| File | Responsibility |
|------|----------------|
| `readme.md` | Project onboarding and directory overview |
| `mod.rs` | Module declaration |
| `caps.rs` | `Cap::Round` zero-segments regression coverage (BUG-236) |
| `colors_desync.rs` | `d3::Line` colors/points independent-`VecDeque` desync regression coverage (BUG-492) |
| `d2_line.rs` | `d2::Line::mesh_update`/`draw` panic-vs-`Result` convention coverage (UX/DX #4) |
| `distance.rs` | Tests related to distance functionality of the basic line |
| `helpers.rs` | `circle_geometry` zero-segments regression coverage (BUG-237) |
| `joins.rs` | `Join::Round/Miter/Bevel` zero-`column_precision` empty-geometry regression coverage (BUG-491) |
| `points.rs` | Tests related to point operation(add, removed) of the basic line |
| `dash.rs` | Tests related to dash configuration (pattern, offset, toggle) of the line |
| `join_tangent.rs` | Rust port of the 5 `.vert` files' shared join-tangent formula; degenerate-cusp NaN regression coverage (BUG-158) |
