//@ name: d2_sdf_ring
//@ description: Unsigned distance from a 2D point to a circle line of the given radius.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_ring(p: vec2f, radius: f32) -> f32
//@ export: fn d2_sdf_ring_preview(p: vec2f, radius: f32) -> f32
//@ param: radius argument f32 range(0.05, 0.45)

fn d2_sdf_ring( p : vec2f, radius : f32 ) -> f32
{
  // Zero exactly on the circle line, growing on both sides — the natural
  // input for stroked rings and orbit lines ( there is no "inside" ).
  return abs( length( p ) - radius );
}

fn d2_sdf_ring_preview( p : vec2f, radius : f32 ) -> f32
{
  return d2_sdf_ring( p, radius );
}
