//@ name: d2_sdf_round_box
//@ description: Signed distance from a 2D point to a box with rounded corners of radius r.
//@ tags: category:sdf, dim:2d
//@ depends_on: d2_sdf_box
//@ export: fn d2_sdf_round_box(p: vec2f, half_extents: vec2f, r: f32) -> f32
//@ export: fn d2_sdf_round_box_preview(p: vec2f) -> f32

fn d2_sdf_round_box( p : vec2f, half_extents : vec2f, r : f32 ) -> f32
{
  // Shrinking the box by r and subtracting r after is exactly equivalent to
  // the closed-form rounded-box formula — see d2_sdf_box for the base shape.
  return d2_sdf_box( p, half_extents - vec2f( r, r ) ) - r;
}

fn d2_sdf_round_box_preview( p : vec2f ) -> f32
{
  return d2_sdf_round_box( p, vec2f( 0.28, 0.18 ), 0.06 );
}
