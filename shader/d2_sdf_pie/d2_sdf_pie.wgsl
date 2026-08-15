//@ name: d2_sdf_pie
//@ description: Signed distance from a 2D point to a pie/wedge slice given by sin/cos of its half-aperture.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_pie(p: vec2f, sc: vec2f, r: f32) -> f32
//@ export: fn d2_sdf_pie_preview(p: vec2f) -> f32

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

fn d2_sdf_pie_preview( p : vec2f ) -> f32
{
  return d2_sdf_pie( p, vec2f( 0.7071, 0.7071 ), 0.3 );
}
