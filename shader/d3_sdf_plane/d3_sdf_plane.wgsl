//@ name: d3_sdf_plane
//@ description: Signed distance from a 3D point to an infinite plane with unit normal n, offset h from the origin.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_plane(p: vec3f, n: vec3f, h: f32) -> f32
//@ export: fn d3_sdf_plane_preview(p: vec2f, offset: f32, z_slice: f32) -> f32
//@ param: offset argument f32 range(-0.3, 0.3)
//@ param: z_slice argument f32 range(-0.3, 0.3)

fn d3_sdf_plane( p : vec3f, n : vec3f, h : f32 ) -> f32
{
  // n must already be normalized — this chunk does not normalize it.
  return dot( p, n ) + h;
}

fn d3_sdf_plane_preview( p : vec2f, offset : f32, z_slice : f32 ) -> f32
{
  // Normal stays fixed at vec3f(0,1,0): the wrapper only tunes offset and
  // slice depth, since independently tunable components could denormalize n.
  return d3_sdf_plane( vec3f( p, z_slice ), vec3f( 0.0, 1.0, 0.0 ), offset );
}
