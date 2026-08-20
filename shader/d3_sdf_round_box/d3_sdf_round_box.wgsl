//@ name: d3_sdf_round_box
//@ description: Signed distance from a 3D point to a box with rounded edges of radius r.
//@ tags: category:sdf, dim:3d
//@ depends_on: d3_sdf_box
//@ export: fn d3_sdf_round_box(p: vec3f, half_extents: vec3f, r: f32) -> f32
//@ export: fn d3_sdf_round_box_preview(p: vec2f, half_extent_x: f32, half_extent_y: f32, half_extent_z: f32, round_radius: f32, z_slice: f32) -> f32
//@ param: half_extent_x argument f32 range(0.1, 0.45)
//@ param: half_extent_y argument f32 range(0.1, 0.45)
//@ param: half_extent_z argument f32 range(0.1, 0.45)
//@ param: round_radius argument f32 range(0.0, 0.1)
//@ param: z_slice argument f32 range(-0.3, 0.3)

fn d3_sdf_round_box( p : vec3f, half_extents : vec3f, r : f32 ) -> f32
{
  // Same shrink-and-offset trick as d2_sdf_round_box, one dimension up.
  return d3_sdf_box( p, half_extents - vec3f( r, r, r ) ) - r;
}

fn d3_sdf_round_box_preview( p : vec2f, half_extent_x : f32, half_extent_y : f32, half_extent_z : f32, round_radius : f32, z_slice : f32 ) -> f32
{
  return d3_sdf_round_box( vec3f( p, z_slice ), vec3f( half_extent_x, half_extent_y, half_extent_z ), round_radius );
}
