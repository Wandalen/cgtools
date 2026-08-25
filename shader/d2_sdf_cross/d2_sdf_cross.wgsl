//@ name: d2_sdf_cross
//@ description: Signed distance from a 2D point to a plus/cross shape of the given half-extents and corner radius r.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_cross(p: vec2f, half_extents: vec2f, r: f32) -> f32
//@ export: fn d2_sdf_cross_preview(p: vec2f, half_extent_x: f32, half_extent_y: f32, corner_radius: f32) -> f32
//@ param: half_extent_x argument f32 range(0.1, 0.45)
//@ param: half_extent_y argument f32 range(0.02, 0.2)
//@ param: corner_radius argument f32 range(0.0, 0.08)

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

fn d2_sdf_cross_preview( p : vec2f, half_extent_x : f32, half_extent_y : f32, corner_radius : f32 ) -> f32
{
  return d2_sdf_cross( p, vec2f( half_extent_x, half_extent_y ), corner_radius );
}
