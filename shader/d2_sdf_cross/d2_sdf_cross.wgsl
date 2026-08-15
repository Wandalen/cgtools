//@ name: d2_sdf_cross
//@ description: Signed distance from a 2D point to a plus/cross shape of the given half-extents and corner radius r.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_cross(p: vec2f, half_extents: vec2f, r: f32) -> f32
//@ export: fn d2_sdf_cross_preview(p: vec2f) -> f32

fn d2_sdf_cross( p_in : vec2f, half_extents : vec2f, r : f32 ) -> f32
{
  var p = abs( p_in );
  if( p.y > p.x )
  {
    p = p.yx;
  }
  let q = p - half_extents;
  let k = max( q.y, q.x );
  var w : vec2f;
  if( k > 0.0 )
  {
    w = q;
  }
  else
  {
    w = vec2f( half_extents.y - p.x, -k );
  }
  return sign( k ) * length( max( w, vec2f( 0.0 ) ) ) + r;
}

fn d2_sdf_cross_preview( p : vec2f ) -> f32
{
  return d2_sdf_cross( p, vec2f( 0.28, 0.09 ), 0.02 );
}
