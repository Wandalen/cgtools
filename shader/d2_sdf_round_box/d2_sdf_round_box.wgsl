//@ name: d2_sdf_round_box
//@ description: Signed distance from a 2D point to a box with rounded corners of radius r.
//@ tags: category:sdf, dim:2d
//@ depends_on: d2_sdf_box
//@ export: fn d2_sdf_round_box(p: vec2f, half_extents: vec2f, r: f32) -> f32
//@ export: fn d2_sdf_round_box_preview(p: vec2f, half_extent_x: f32, half_extent_y: f32, round_radius: f32) -> f32
//@ param: half_extent_x argument f32 range(0.1, 0.45)
//@ param: half_extent_y argument f32 range(0.05, 0.35)
//@ param: round_radius argument f32 range(0.0, 0.15)

fn d2_sdf_round_box( p : vec2f, half_extents : vec2f, r : f32 ) -> f32
{
  // Shrinking the box by r and subtracting r after is exactly equivalent to
  // the closed-form rounded-box formula — see d2_sdf_box for the base shape.
  return d2_sdf_box( p, half_extents - vec2f( r, r ) ) - r;
}

fn d2_sdf_round_box_preview( p : vec2f, half_extent_x : f32, half_extent_y : f32, round_radius : f32 ) -> f32
{
  return d2_sdf_round_box( p, vec2f( half_extent_x, half_extent_y ), round_radius );
}
