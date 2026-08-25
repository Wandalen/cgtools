//@ name: rot2
//@ description: 2D rotation matrix for the given angle in radians, counterclockwise.
//@ tags: category:transform
//@ depends_on:
//@ export: fn rot2(angle: f32) -> mat2x2f
//@ export: fn rot2_preview(p: vec2f, angle: f32, stripe_frequency: f32) -> f32
//@ param: angle argument f32 range(0.0, 6.283)
//@ param: stripe_frequency argument f32 range(5.0, 100.0)

fn rot2( angle : f32 ) -> mat2x2f
{
  let s = sin( angle );
  let c = cos( angle );
  // Column-major : columns ( c, s ) and ( -s, c ) — counterclockwise for
  // column vectors under `rot * v` in the usual y-up convention.
  return mat2x2f( c, s, -s, c );
}

fn rot2_preview( p : vec2f, angle : f32, stripe_frequency : f32 ) -> f32
{
  let q = rot2( angle ) * p;
  return 0.5 + 0.5 * cos( q.x * stripe_frequency );
}
