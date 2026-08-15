//@ name: d3_sdf_hex_prism
//@ description: Signed distance from a 3D point to a hexagonal prism of circumradius h.x and half-depth h.y, axis along z.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_hex_prism(p: vec3f, h: vec2f) -> f32
//@ export: fn d3_sdf_hex_prism_preview(p: vec2f) -> f32

fn d3_sdf_hex_prism( p_in : vec3f, h : vec2f ) -> f32
{
  let k = vec3f( -0.8660254, 0.5, 0.57735 );
  var p = abs( p_in );
  let xy_folded = p.xy - 2.0 * min( dot( k.xy, p.xy ), 0.0 ) * k.xy;
  p = vec3f( xy_folded, p.z );
  let d = vec2f
  (
    length( p.xy - vec2f( clamp( p.x, -k.z * h.x, k.z * h.x ), h.x ) ) * sign( p.y - h.x ),
    p.z - h.y
  );
  return min( max( d.x, d.y ), 0.0 ) + length( max( d, vec2f( 0.0 ) ) );
}

fn d3_sdf_hex_prism_preview( p : vec2f ) -> f32
{
  return d3_sdf_hex_prism( vec3f( p, 0.0 ), vec2f( 0.26, 0.2 ) );
}
