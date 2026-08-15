//@ name: d2_sdf_hexagon
//@ description: Signed distance from a 2D point to a regular hexagon of the given circumradius.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_hexagon(p: vec2f, r: f32) -> f32
//@ export: fn d2_sdf_hexagon_preview(p: vec2f) -> f32

fn d2_sdf_hexagon( p_in : vec2f, r : f32 ) -> f32
{
  let k = vec3f( -0.866025404, 0.5, 0.577350269 );
  var p = abs( p_in );
  p -= 2.0 * min( dot( k.xy, p ), 0.0 ) * k.xy;
  p -= vec2f( clamp( p.x, -k.z * r, k.z * r ), r );
  return length( p ) * sign( p.y );
}

fn d2_sdf_hexagon_preview( p : vec2f ) -> f32
{
  return d2_sdf_hexagon( p, 0.26 );
}
