//@ name: sdf_op_subtract
//@ description: Sharp subtraction of shape d1 from shape d2 (carves d1 out of d2).
//@ tags: category:sdf, technique:operator
//@ depends_on: d2_sdf_circle, d2_sdf_box
//@ export: fn sdf_op_subtract(d1: f32, d2: f32) -> f32
//@ export: fn sdf_op_subtract_preview(p: vec2f, circle_offset_x: f32, circle_radius: f32, box_offset_x: f32, box_half_extent: f32) -> f32
//@ param: circle_offset_x argument f32 range(-0.3, 0.3)
//@ param: circle_radius argument f32 range(0.05, 0.35)
//@ param: box_offset_x argument f32 range(-0.3, 0.3)
//@ param: box_half_extent argument f32 range(0.05, 0.3)

fn sdf_op_subtract( d1 : f32, d2 : f32 ) -> f32
{
  return max( -d1, d2 );
}

fn sdf_op_subtract_preview( p : vec2f, circle_offset_x : f32, circle_radius : f32, box_offset_x : f32, box_half_extent : f32 ) -> f32
{
  let d1 = d2_sdf_circle( p - vec2f( circle_offset_x, 0.0 ), circle_radius );
  let d2 = d2_sdf_box( p - vec2f( box_offset_x, 0.0 ), vec2f( box_half_extent, box_half_extent ) );
  return sdf_op_subtract( d1, d2 );
}
