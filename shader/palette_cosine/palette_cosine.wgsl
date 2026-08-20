//@ name: palette_cosine
//@ description: Cosine color palette: a + b*cos(2pi*(c*t + d)), the classic 4-parameter gradient.
//@ tags: category:color
//@ depends_on:
//@ export: fn palette_cosine(t: f32, a: vec3f, b: vec3f, c: vec3f, d: vec3f) -> vec3f
//@ export: fn palette_cosine_preview(p: vec2f) -> vec3f

fn palette_cosine( t : f32, a : vec3f, b : vec3f, c : vec3f, d : vec3f ) -> vec3f
{
  // a = base color, b = amplitude, c = frequency, d = per-channel phase.
  // One cheap trig call replaces multi-stop gradient mix chains.
  return a + b * cos( 6.28318530718 * ( c * t + d ) );
}

fn palette_cosine_preview( p : vec2f ) -> vec3f
{
  // Fixed canonical rainbow parameterization ( see readme.md's
  // Visualization section ) -- the point of this demo is showing three
  // channels visibly separated by phase, so the spread must not be a
  // tunable that can collapse to a single shared value.
  return palette_cosine( p.x, vec3f( 0.5 ), vec3f( 0.5 ), vec3f( 1.0 ), vec3f( 0.0, 0.33, 0.67 ) );
}
