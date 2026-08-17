//@ name: d3_sdf_sphere
//@ description: Signed distance from a 3D point to a sphere of the given radius.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_sphere(p: vec3f, radius: f32) -> f32
//@ export: fn d3_sdf_sphere_preview(p: vec2f, radius: f32, z_slice: f32) -> f32
//@ param: radius argument f32 range(0.05, 0.45)
//@ param: z_slice argument f32 range(-0.3, 0.3)

fn d3_sdf_sphere( p : vec3f, radius : f32 ) -> f32
{
  return length( p ) - radius;
}

fn d3_sdf_sphere_preview( p : vec2f, radius : f32, z_slice : f32 ) -> f32
{
  return d3_sdf_sphere( vec3f( p, z_slice ), radius );
}
