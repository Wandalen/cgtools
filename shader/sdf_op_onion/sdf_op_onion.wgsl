//@ name: sdf_op_onion
//@ description: Turns a filled signed distance field into a hollow shell of the given thickness.
//@ tags: category:sdf, technique:operator
//@ depends_on: d2_sdf_box
//@ export: fn sdf_op_onion(d: f32, thickness: f32) -> f32
//@ export: fn sdf_op_onion_preview(p: vec2f, box_half_extent_x: f32, box_half_extent_y: f32, thickness: f32) -> f32
//@ param: box_half_extent_x argument f32 range(0.1, 0.4)
//@ param: box_half_extent_y argument f32 range(0.1, 0.35)
//@ param: thickness argument f32 range(0.01, 0.1)

fn sdf_op_onion( d : f32, thickness : f32 ) -> f32
{
  return abs( d ) - thickness;
}

fn sdf_op_onion_preview( p : vec2f, box_half_extent_x : f32, box_half_extent_y : f32, thickness : f32 ) -> f32
{
  return sdf_op_onion( d2_sdf_box( p, vec2f( box_half_extent_x, box_half_extent_y ) ), thickness );
}
