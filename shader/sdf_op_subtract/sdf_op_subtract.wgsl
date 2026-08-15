//@ name: sdf_op_subtract
//@ description: Sharp subtraction of shape d1 from shape d2 (carves d1 out of d2).
//@ tags: category:sdf, technique:operator
//@ depends_on: d2_sdf_circle, d2_sdf_box
//@ export: fn sdf_op_subtract(d1: f32, d2: f32) -> f32
//@ export: fn sdf_op_subtract_preview(p: vec2f) -> f32

fn sdf_op_subtract( d1 : f32, d2 : f32 ) -> f32
{
  return max( -d1, d2 );
}

fn sdf_op_subtract_preview( p : vec2f ) -> f32
{
  let d1 = d2_sdf_circle( p - vec2f( -0.13, 0.0 ), 0.22 );
  let d2 = d2_sdf_box( p - vec2f( 0.15, 0.0 ), vec2f( 0.16, 0.16 ) );
  return sdf_op_subtract( d1, d2 );
}
