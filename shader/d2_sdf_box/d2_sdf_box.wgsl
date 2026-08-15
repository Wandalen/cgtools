//@ name: d2_sdf_box
//@ description: Signed distance from a 2D point to an axis-aligned box of the given half-extents.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_box(p: vec2f, half_extents: vec2f) -> f32
//@ export: fn d2_sdf_box_preview(p: vec2f) -> f32

fn d2_sdf_box( p : vec2f, half_extents : vec2f ) -> f32
{
  let d = abs( p ) - half_extents;
  return length( max( d, vec2f( 0.0 ) ) ) + min( max( d.x, d.y ), 0.0 );
}

fn d2_sdf_box_preview( p : vec2f ) -> f32
{
  return d2_sdf_box( p, vec2f( 0.28, 0.18 ) );
}
