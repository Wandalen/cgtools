//@ name: sdf_op_round
//@ description: Rounds a signed distance field's corners by shrinking the shape and offsetting outward by r.
//@ tags: category:sdf, technique:operator
//@ depends_on: d2_sdf_box
//@ export: fn sdf_op_round(d: f32, r: f32) -> f32
//@ export: fn sdf_op_round_preview(p: vec2f) -> f32

fn sdf_op_round( d : f32, r : f32 ) -> f32
{
  return d - r;
}

fn sdf_op_round_preview( p : vec2f ) -> f32
{
  return sdf_op_round( d2_sdf_box( p, vec2f( 0.22, 0.22 ) ), 0.08 );
}
