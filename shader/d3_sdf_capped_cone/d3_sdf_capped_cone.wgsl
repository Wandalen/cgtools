//@ name: d3_sdf_capped_cone
//@ description: Signed distance from a 3D point to a capped cone of half-height h between radii r1 (bottom) and r2 (top), axis along y.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_capped_cone(p: vec3f, h: f32, r1: f32, r2: f32) -> f32
//@ export: fn d3_sdf_capped_cone_preview(p: vec2f, half_height: f32, radius_bottom: f32, radius_top: f32, z_slice: f32) -> f32
//@ param: half_height argument f32 range(0.05, 0.4)
//@ param: radius_bottom argument f32 range(0.02, 0.4)
//@ param: radius_top argument f32 range(0.0, 0.4)
//@ param: z_slice argument f32 range(-0.3, 0.3)

fn d3_sdf_capped_cone( p : vec3f, h : f32, r1 : f32, r2 : f32 ) -> f32
{
  let q = vec2f( length( p.xz ), p.y );
  let k1 = vec2f( r2, h );
  let k2 = vec2f( r2 - r1, 2.0 * h );
  var r_sel : f32 = r2;
  if( q.y < 0.0 )
  {
    r_sel = r1;
  }
  let ca = vec2f( q.x - min( q.x, r_sel ), abs( q.y ) - h );
  let cb = q - k1 + k2 * clamp( dot( k1 - q, k2 ) / dot( k2, k2 ), 0.0, 1.0 );
  var s : f32 = 1.0;
  if( cb.x < 0.0 && ca.y < 0.0 )
  {
    s = -1.0;
  }
  return s * sqrt( min( dot( ca, ca ), dot( cb, cb ) ) );
}

fn d3_sdf_capped_cone_preview( p : vec2f, half_height : f32, radius_bottom : f32, radius_top : f32, z_slice : f32 ) -> f32
{
  return d3_sdf_capped_cone( vec3f( p, z_slice ), half_height, radius_bottom, radius_top );
}
