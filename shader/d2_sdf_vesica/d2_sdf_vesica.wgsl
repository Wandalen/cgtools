//@ name: d2_sdf_vesica
//@ description: Signed distance from a 2D point to a vesica (lens) shape from two circles of radius r offset by d.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_vesica(p: vec2f, r: f32, d: f32) -> f32
//@ export: fn d2_sdf_vesica_preview(p: vec2f) -> f32

fn d2_sdf_vesica( p_in : vec2f, r : f32, d : f32 ) -> f32
{
  let p = abs( p_in );
  let b = sqrt( r * r - d * d );
  if( ( p.y - b ) * d > p.x * b )
  {
    return length( p - vec2f( 0.0, b ) );
  }
  return length( p - vec2f( -d, 0.0 ) ) - r;
}

fn d2_sdf_vesica_preview( p : vec2f ) -> f32
{
  return d2_sdf_vesica( p, 0.3, 0.15 );
}
