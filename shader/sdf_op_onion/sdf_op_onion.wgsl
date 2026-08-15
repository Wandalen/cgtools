//@ name: sdf_op_onion
//@ description: Turns a filled signed distance field into a hollow shell of the given thickness.
//@ tags: category:sdf, technique:operator
//@ depends_on: d2_sdf_box
//@ export: fn sdf_op_onion(d: f32, thickness: f32) -> f32
//@ export: fn sdf_op_onion_preview(p: vec2f) -> f32

fn sdf_op_onion( d : f32, thickness : f32 ) -> f32
{
  return abs( d ) - thickness;
}

fn sdf_op_onion_preview( p : vec2f ) -> f32
{
  return sdf_op_onion( d2_sdf_box( p, vec2f( 0.26, 0.2 ) ), 0.045 );
}
