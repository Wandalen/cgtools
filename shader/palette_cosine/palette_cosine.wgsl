//@ name: palette_cosine
//@ description: Cosine color palette: a + b*cos(2pi*(c*t + d)), the classic 4-parameter gradient.
//@ tags: category:color
//@ depends_on:
//@ export: fn palette_cosine(t: f32, a: vec3f, b: vec3f, c: vec3f, d: vec3f) -> vec3f
//@ export: fn palette_cosine_preview(p: vec2f, base: f32, amplitude: f32, frequency: f32, phase_r: f32, phase_g: f32, phase_b: f32) -> vec3f
//@ param: base argument f32 range(0.0, 1.0)
//@ param: amplitude argument f32 range(0.0, 1.0)
//@ param: frequency argument f32 range(0.1, 4.0)
//@ param: phase_r argument f32 range(0.0, 1.0)
//@ param: phase_g argument f32 range(0.0, 1.0)
//@ param: phase_b argument f32 range(0.0, 1.0)

fn palette_cosine( t : f32, a : vec3f, b : vec3f, c : vec3f, d : vec3f ) -> vec3f
{
  // a = base color, b = amplitude, c = frequency, d = per-channel phase.
  // One cheap trig call replaces multi-stop gradient mix chains.
  return a + b * cos( 6.28318530718 * ( c * t + d ) );
}

fn palette_cosine_preview( p : vec2f, base : f32, amplitude : f32, frequency : f32, phase_r : f32, phase_g : f32, phase_b : f32 ) -> vec3f
{
  return palette_cosine( p.x, vec3f( base ), vec3f( amplitude ), vec3f( frequency ), vec3f( phase_r, phase_g, phase_b ) );
}
