//@ name: d2_sdf_pie
//@ description: Signed distance from a 2D point to a pie/wedge slice given by sin/cos of its half-aperture.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_pie(p: vec2f, sc: vec2f, r: f32) -> f32
//@ export: fn d2_sdf_pie_preview(p: vec2f, half_aperture: f32, radius: f32) -> f32
//@ param: half_aperture argument f32 range(0.1, 3.0)
//@ param: radius argument f32 range(0.05, 0.45)

fn d2_sdf_pie( p_in : vec2f, sc : vec2f, r : f32 ) -> f32
{
  // sc = ( sin, cos ) of the wedge's half-aperture angle; r = slice radius.
  // Mirrored across x, so the wedge is centered on +y.
  var p = p_in;
  p.x = abs( p.x );
  let l = length( p ) - r;
  let m = length( p - sc * clamp( dot( p, sc ), 0.0, r ) );
  return max( l, m * sign( sc.y * p.x - sc.x * p.y ) );
}

fn d2_sdf_pie_preview( p : vec2f, half_aperture : f32, radius : f32 ) -> f32
{
  return d2_sdf_pie( p, vec2f( sin( half_aperture ), cos( half_aperture ) ), radius );
}
