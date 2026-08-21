# tests

Integration tests for `primitive_generation`, exercising the crate's public API only.

## Responsibility Table

| File | Responsibility |
| ---- | -------------- |
| contours_to_fill_geometry_test.rs | `contours_to_fill_geometry` triangulated-fill generation |
| curve_to_geometry_test.rs | `curve_to_geometry` precondition and geometry generation |
| font_bounding_box_union_test.rs | `Font::max_size` bounding-box union correctness |
| geometry_normal_attribute_test.rs | `AttributesData::normals` population across all generators |
| path_to_points_test.rs | `path_to_points` flattening and closed-path handling |
| primitive_data_test.rs | `primitives_parent_graph_validate` acyclic parent-graph check |
| solid_test.rs | Procedural solid-mesh generators (box, cylinder, torus, icosphere) |
| ufo_font_scale_test.rs | `text::ufo::glyph_rescale_factor` zero-height-guard scaling |
| ufo_glif_point_type_test.rs | `.glif` XML point-type parsing |
| ufo_text_advance_test.rs | `text_to_countour_mesh` two-pass glyph-advance layout |
