//@ name: aa_step
//@ description: Antialiased step via fwidth: a screen-space smoothed threshold (fragment-stage only).
//@ tags: category:antialiasing
//@ depends_on:
//@ export: fn aa_step(edge: f32, x: f32) -> f32
//@ export: fn aa_step_preview(p: vec2f, edge: f32) -> f32
//@ param: edge argument f32 range(0.05, 0.6)

fn aa_step( edge : f32, x : f32 ) -> f32
{
  // The transition band tracks the on-screen derivative of x, so edges stay
  // one pixel wide at any resolution or zoom — unlike a hardcoded epsilon.
  // fwidth is a derivative builtin : callable from fragment-stage code only.
  let w = fwidth( x );
  return smoothstep( edge - w, edge + w, x );
}

fn aa_step_preview( p : vec2f, edge : f32 ) -> f32
{
  return 1.0 - aa_step( edge, length( p ) );
}
