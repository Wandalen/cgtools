//@ name: glow
//@ description: Analytic radial falloff: 1 at distance 0 fading to 0 at the given radius.
//@ tags: category:shading
//@ depends_on:
//@ export: fn glow(d: f32, radius: f32) -> f32
//@ export: fn glow_preview(p: vec2f, radius: f32) -> f32
//@ param: radius argument f32 range(0.05, 1.0)

fn glow( d : f32, radius : f32 ) -> f32
{
  // Smooth-edged falloff over a distance — the halo/bloom-substitute idiom
  // for single-pass scenes with no render-target blur infrastructure.
  return 1.0 - smoothstep( 0.0, radius, d );
}

fn glow_preview( p : vec2f, radius : f32 ) -> f32
{
  return glow( length( p ), radius );
}
