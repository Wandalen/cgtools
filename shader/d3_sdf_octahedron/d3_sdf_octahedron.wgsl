//@ name: d3_sdf_octahedron
//@ description: Signed distance from a 3D point to an octahedron of the given size, exact (not bound).
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_octahedron(p: vec3f, s: f32) -> f32
//@ export: fn d3_sdf_octahedron_preview(p: vec2f) -> f32

fn d3_sdf_octahedron( p_in : vec3f, s : f32 ) -> f32
{
  let p = abs( p_in );
  let m = p.x + p.y + p.z - s;
  var q : vec3f;
  if( 3.0 * p.x < m )
  {
    q = p.xyz;
  }
  else if( 3.0 * p.y < m )
  {
    q = p.yzx;
  }
  else if( 3.0 * p.z < m )
  {
    q = p.zxy;
  }
  else
  {
    return m * 0.57735027;
  }
  let k = clamp( 0.5 * ( q.z - q.y + s ), 0.0, s );
  return length( vec3f( q.x, q.y - s + k, q.z - k ) );
}

fn d3_sdf_octahedron_preview( p : vec2f ) -> f32
{
  return d3_sdf_octahedron( vec3f( p, 0.0 ), 0.32 );
}
