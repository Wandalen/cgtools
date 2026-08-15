//@ name: d3_sdf_round_box
//@ description: Signed distance from a 3D point to a box with rounded edges of radius r.
//@ tags: category:sdf, dim:3d
//@ depends_on: d3_sdf_box
//@ export: fn d3_sdf_round_box(p: vec3f, half_extents: vec3f, r: f32) -> f32
//@ export: fn d3_sdf_round_box_preview(p: vec2f) -> f32

fn d3_sdf_round_box( p : vec3f, half_extents : vec3f, r : f32 ) -> f32
{
  // Same shrink-and-offset trick as d2_sdf_round_box, one dimension up.
  return d3_sdf_box( p, half_extents - vec3f( r, r, r ) ) - r;
}

fn d3_sdf_round_box_preview( p : vec2f ) -> f32
{
  return d3_sdf_round_box( vec3f( p, 0.0 ), vec3f( 0.28, 0.18, 0.22 ), 0.06 );
}
