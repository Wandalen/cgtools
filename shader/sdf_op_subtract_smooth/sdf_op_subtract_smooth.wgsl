//@ name: sdf_op_subtract_smooth
//@ description: Smoothly blended subtraction of shape d1 from shape d2 with blend radius k.
//@ tags: category:sdf, technique:operator
//@ depends_on: d2_sdf_circle, d2_sdf_box
//@ export: fn sdf_op_subtract_smooth(d1: f32, d2: f32, k: f32) -> f32
//@ export: fn sdf_op_subtract_smooth_preview(p: vec2f) -> f32

fn sdf_op_subtract_smooth( d1 : f32, d2 : f32, k : f32 ) -> f32
{
  let h = clamp( 0.5 - 0.5 * ( d2 + d1 ) / k, 0.0, 1.0 );
  return mix( d2, -d1, h ) + k * h * ( 1.0 - h );
}

fn sdf_op_subtract_smooth_preview( p : vec2f ) -> f32
{
  let d1 = d2_sdf_circle( p - vec2f( -0.13, 0.0 ), 0.22 );
  let d2 = d2_sdf_box( p - vec2f( 0.15, 0.0 ), vec2f( 0.16, 0.16 ) );
  return sdf_op_subtract_smooth( d1, d2, 0.05 );
}
