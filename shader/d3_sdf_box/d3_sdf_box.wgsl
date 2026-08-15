//@ name: d3_sdf_box
//@ description: Signed distance from a 3D point to an axis-aligned box of the given half-extents.
//@ tags: category:sdf, dim:3d
//@ depends_on:
//@ export: fn d3_sdf_box(p: vec3f, half_extents: vec3f) -> f32
//@ export: fn d3_sdf_box_preview(p: vec2f) -> f32

fn d3_sdf_box( p : vec3f, half_extents : vec3f ) -> f32
{
  let q = abs( p ) - half_extents;
  return length( max( q, vec3f( 0.0 ) ) ) + min( max( q.x, max( q.y, q.z ) ), 0.0 );
}

fn d3_sdf_box_preview( p : vec2f ) -> f32
{
  return d3_sdf_box( vec3f( p, 0.0 ), vec3f( 0.28, 0.18, 0.22 ) );
}
