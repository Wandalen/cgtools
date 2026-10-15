//@ name: d3_sdf_box
//@ description: Signed distance from a 3D point to an axis-aligned box of the given half-extents.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_box(p: vec3f, half_extents: vec3f) -> f32
//@ export: fn d3_sdf_box_preview(p: vec2f, half_extent_x: f32, half_extent_y: f32, half_extent_z: f32, z_slice: f32) -> f32
//@ param: half_extent_x argument f32 range(0.05, 0.45)
//@ param: half_extent_y argument f32 range(0.05, 0.45)
//@ param: half_extent_z argument f32 range(0.05, 0.45)
//@ param: z_slice argument f32 range(-0.3, 0.3)

fn d3_sdf_box( p : vec3f, half_extents : vec3f ) -> f32
{
  let q = abs( p ) - half_extents;
  return length( max( q, vec3f( 0.0 ) ) ) + min( max( q.x, max( q.y, q.z ) ), 0.0 );
}

fn d3_sdf_box_preview( p : vec2f, half_extent_x : f32, half_extent_y : f32, half_extent_z : f32, z_slice : f32 ) -> f32
{
  return d3_sdf_box( vec3f( p, z_slice ), vec3f( half_extent_x, half_extent_y, half_extent_z ) );
}
