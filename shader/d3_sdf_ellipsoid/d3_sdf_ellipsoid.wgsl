//@ name: d3_sdf_ellipsoid
//@ description: Signed distance bound from a 3D point to an ellipsoid of the given per-axis radii (not exact).
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_ellipsoid(p: vec3f, r: vec3f) -> f32
//@ export: fn d3_sdf_ellipsoid_preview(p: vec2f) -> f32

fn d3_sdf_ellipsoid( p : vec3f, r : vec3f ) -> f32
{
  // A distance bound, not an exact Euclidean distance — see Nuances.
  let k0 = length( p / r );
  let k1 = length( p / ( r * r ) );
  return k0 * ( k0 - 1.0 ) / k1;
}

fn d3_sdf_ellipsoid_preview( p : vec2f ) -> f32
{
  return d3_sdf_ellipsoid( vec3f( p, 0.0 ), vec3f( 0.32, 0.18, 0.24 ) );
}
