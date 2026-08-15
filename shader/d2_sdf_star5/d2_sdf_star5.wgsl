//@ name: d2_sdf_star5
//@ description: Signed distance from a 2D point to a 5-pointed star of outer radius r and inner-radius factor rf.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_star5(p: vec2f, r: f32, rf: f32) -> f32
//@ export: fn d2_sdf_star5_preview(p: vec2f) -> f32

fn d2_sdf_star5( p_in : vec2f, r : f32, rf : f32 ) -> f32
{
  let k1 = vec2f( 0.809016994375, -0.587785252292 );
  let k2 = vec2f( -k1.x, k1.y );
  var p = p_in;
  p.x = abs( p.x );
  p -= 2.0 * max( dot( k1, p ), 0.0 ) * k1;
  p -= 2.0 * max( dot( k2, p ), 0.0 ) * k2;
  p.x = abs( p.x );
  p.y -= r;
  let ba = rf * vec2f( -k1.y, k1.x ) - vec2f( 0.0, 1.0 );
  let h = clamp( dot( p, ba ) / dot( ba, ba ), 0.0, r );
  return length( p - ba * h ) * sign( p.y * ba.x - p.x * ba.y );
}

fn d2_sdf_star5_preview( p : vec2f ) -> f32
{
  return d2_sdf_star5( p, 0.3, 0.5 );
}
