//@ name: d3_sdf_ellipsoid
//@ description: Signed distance bound from a 3D point to an ellipsoid of the given per-axis radii (not exact).
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_ellipsoid(p: vec3f, r: vec3f) -> f32
//@ export: fn d3_sdf_ellipsoid_preview(p: vec2f, radius_x: f32, radius_y: f32, radius_z: f32, z_slice: f32) -> f32
//@ param: radius_x argument f32 range(0.05, 0.45)
//@ param: radius_y argument f32 range(0.05, 0.45)
//@ param: radius_z argument f32 range(0.05, 0.45)
//@ param: z_slice argument f32 range(-0.3, 0.3)

fn d3_sdf_ellipsoid( p : vec3f, r : vec3f ) -> f32
{
  // A distance bound, not an exact Euclidean distance — see Nuances.
  let k0 = length( p / r );
  let k1 = length( p / ( r * r ) );
  return k0 * ( k0 - 1.0 ) / k1;
}

fn d3_sdf_ellipsoid_preview( p : vec2f, radius_x : f32, radius_y : f32, radius_z : f32, z_slice : f32 ) -> f32
{
  return d3_sdf_ellipsoid( vec3f( p, z_slice ), vec3f( radius_x, radius_y, radius_z ) );
}
