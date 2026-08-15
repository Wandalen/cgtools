//@ name: d2_sdf_arc
//@ description: Unsigned distance from a 2D point to a ring arc given by sin/cos of its half-aperture.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_arc(p: vec2f, sc: vec2f, ra: f32, rb: f32) -> f32
//@ export: fn d2_sdf_arc_preview(p: vec2f) -> f32

fn d2_sdf_arc( p_in : vec2f, sc : vec2f, ra : f32, rb : f32 ) -> f32
{
  // sc = ( sin, cos ) of the arc's half-aperture angle; ra = arc radius;
  // rb = stroke half-thickness. Mirrored across x, so the arc opens along +y.
  var p = p_in;
  p.x = abs( p.x );
  var d : f32;
  if( sc.y * p.x > sc.x * p.y )
  {
    d = length( p - sc * ra );
  }
  else
  {
    d = abs( length( p ) - ra );
  }
  return d - rb;
}

fn d2_sdf_arc_preview( p : vec2f ) -> f32
{
  return d2_sdf_arc( p, vec2f( 1.0, 0.0 ), 0.28, 0.05 );
}
