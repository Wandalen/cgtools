//@ name: d3_sdf_round_cone
//@ description: Signed distance from a 3D point to a round cone (swept sphere) of height h between radii r1 (bottom) and r2 (top), axis along y.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_round_cone(p: vec3f, r1: f32, r2: f32, h: f32) -> f32
//@ export: fn d3_sdf_round_cone_preview(p: vec2f) -> f32

fn d3_sdf_round_cone( p : vec3f, r1 : f32, r2 : f32, h : f32 ) -> f32
{
  let q = vec2f( length( p.xz ), p.y );
  let b = ( r1 - r2 ) / h;
  let a = sqrt( 1.0 - b * b );
  let k = dot( q, vec2f( -b, a ) );
  if( k < 0.0 )
  {
    return length( q ) - r1;
  }
  if( k > a * h )
  {
    return length( q - vec2f( 0.0, h ) ) - r2;
  }
  return dot( q, vec2f( a, b ) ) - r1;
}

fn d3_sdf_round_cone_preview( p : vec2f ) -> f32
{
  return d3_sdf_round_cone( vec3f( p, 0.0 ), 0.22, 0.08, 0.36 );
}
