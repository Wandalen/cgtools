//@ name: d2_sdf_equilateral_triangle
//@ description: Signed distance from a 2D point to an equilateral triangle of the given circumradius, apex up.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_equilateral_triangle(p: vec2f, r: f32) -> f32
//@ export: fn d2_sdf_equilateral_triangle_preview(p: vec2f) -> f32

fn d2_sdf_equilateral_triangle( p_in : vec2f, r : f32 ) -> f32
{
  let k = sqrt( 3.0 );
  var p = p_in;
  p.x = abs( p.x ) - r;
  p.y = p.y + r / k;
  if( p.x + k * p.y > 0.0 )
  {
    p = vec2f( p.x - k * p.y, -k * p.x - p.y ) / 2.0;
  }
  p.x -= clamp( p.x, -2.0 * r, 0.0 );
  return -length( p ) * sign( p.y );
}

fn d2_sdf_equilateral_triangle_preview( p : vec2f ) -> f32
{
  return d2_sdf_equilateral_triangle( p, 0.28 );
}
