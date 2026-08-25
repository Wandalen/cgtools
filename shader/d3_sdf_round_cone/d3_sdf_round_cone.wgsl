//@ name: d3_sdf_round_cone
//@ description: Signed distance from a 3D point to a round cone (swept sphere) of height h between radii r1 (bottom) and r2 (top), axis along y.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_round_cone(p: vec3f, r1: f32, r2: f32, h: f32) -> f32
//@ export: fn d3_sdf_round_cone_preview(p: vec2f, radius_bottom: f32, radius_top: f32, height: f32, z_slice: f32) -> f32
//@ param: radius_bottom argument f32 range(0.05, 0.3)
//@ param: radius_top argument f32 range(0.02, 0.2)
//@ param: height argument f32 range(0.35, 0.5)
//@ param: z_slice argument f32 range(-0.3, 0.3)

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

fn d3_sdf_round_cone_preview( p : vec2f, radius_bottom : f32, radius_top : f32, height : f32, z_slice : f32 ) -> f32
{
  return d3_sdf_round_cone( vec3f( p, z_slice ), radius_bottom, radius_top, height );
}
