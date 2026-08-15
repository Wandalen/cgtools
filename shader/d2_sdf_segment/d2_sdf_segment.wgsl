//@ name: d2_sdf_segment
//@ description: Unsigned distance from a 2D point to the line segment between two endpoints.
//@ tags: category:sdf, dim:2d
//@ depends_on:
//@ export: fn d2_sdf_segment(p: vec2f, a: vec2f, b: vec2f) -> f32
//@ export: fn d2_sdf_segment_preview(p: vec2f) -> f32

fn d2_sdf_segment( p : vec2f, a : vec2f, b : vec2f ) -> f32
{
  let pa = p - a;
  let ba = b - a;
  let h = clamp( dot( pa, ba ) / dot( ba, ba ), 0.0, 1.0 );
  return length( pa - ba * h );
}

fn d2_sdf_segment_preview( p : vec2f ) -> f32
{
  return d2_sdf_segment( p, vec2f( -0.25, -0.1 ), vec2f( 0.25, 0.15 ) );
}
