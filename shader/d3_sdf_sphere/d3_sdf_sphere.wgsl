//@ name: d3_sdf_sphere
//@ description: Signed distance from a 3D point to a sphere of the given radius.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_sphere(p: vec3f, radius: f32) -> f32
//@ export: fn d3_sdf_sphere_preview(p: vec2f) -> f32

fn d3_sdf_sphere( p : vec3f, radius : f32 ) -> f32
{
  return length( p ) - radius;
}

fn d3_sdf_sphere_preview( p : vec2f ) -> f32
{
  return d3_sdf_sphere( vec3f( p, 0.0 ), 0.28 );
}
