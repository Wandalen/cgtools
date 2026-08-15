//@ name: d3_sdf_plane
//@ description: Signed distance from a 3D point to an infinite plane with unit normal n, offset h from the origin.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_plane(p: vec3f, n: vec3f, h: f32) -> f32
//@ export: fn d3_sdf_plane_preview(p: vec2f) -> f32

fn d3_sdf_plane( p : vec3f, n : vec3f, h : f32 ) -> f32
{
  // n must already be normalized — this chunk does not normalize it.
  return dot( p, n ) + h;
}

fn d3_sdf_plane_preview( p : vec2f ) -> f32
{
  return d3_sdf_plane( vec3f( p, 0.0 ), vec3f( 0.0, 1.0, 0.0 ), 0.0 );
}
