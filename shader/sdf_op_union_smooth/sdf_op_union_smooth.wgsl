//@ name: sdf_op_union_smooth
//@ description: Smoothly blended union of two signed distances with blend radius k.
//@ tags: category:sdf, technique:operator
//@ depends_on: d2_sdf_circle, d2_sdf_box
//@ export: fn sdf_op_union_smooth(d1: f32, d2: f32, k: f32) -> f32
//@ export: fn sdf_op_union_smooth_preview(p: vec2f, circle_offset_x: f32, circle_radius: f32, box_offset_x: f32, box_half_extent: f32, blend_radius: f32) -> f32
//@ param: circle_offset_x argument f32 range(-0.3, 0.3)
//@ param: circle_radius argument f32 range(0.05, 0.35)
//@ param: box_offset_x argument f32 range(-0.3, 0.3)
//@ param: box_half_extent argument f32 range(0.05, 0.3)
//@ param: blend_radius argument f32 range(0.01, 0.15)

fn sdf_op_union_smooth( d1 : f32, d2 : f32, k : f32 ) -> f32
{
  let h = clamp( 0.5 + 0.5 * ( d2 - d1 ) / k, 0.0, 1.0 );
  return mix( d2, d1, h ) - k * h * ( 1.0 - h );
}

fn sdf_op_union_smooth_preview( p : vec2f, circle_offset_x : f32, circle_radius : f32, box_offset_x : f32, box_half_extent : f32, blend_radius : f32 ) -> f32
{
  let d1 = d2_sdf_circle( p - vec2f( circle_offset_x, 0.0 ), circle_radius );
  let d2 = d2_sdf_box( p - vec2f( box_offset_x, 0.0 ), vec2f( box_half_extent, box_half_extent ) );
  return sdf_op_union_smooth( d1, d2, blend_radius );
}
