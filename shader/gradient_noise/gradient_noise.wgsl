//@ name: gradient_noise
//@ description: Quintic-faded gradient (Perlin) noise at a 2D point, roughly in [-0.7, 0.7].
//@ tags: category:noise, technique:gradient
//@ depends_on: hash22
//@ export: fn gradient_noise(p: vec2f) -> f32

fn gradient_noise( p : vec2f ) -> f32
{
  let i = floor( p );
  let f = fract( p );
  // Per-corner pseudo-random gradients in [-1, 1]^2, dotted with the
  // offset from that corner to the sample point.
  let ga = hash22( i ) * 2.0 - vec2f( 1.0 );
  let gb = hash22( i + vec2f( 1.0, 0.0 ) ) * 2.0 - vec2f( 1.0 );
  let gc = hash22( i + vec2f( 0.0, 1.0 ) ) * 2.0 - vec2f( 1.0 );
  let gd = hash22( i + vec2f( 1.0, 1.0 ) ) * 2.0 - vec2f( 1.0 );
  let va = dot( ga, f );
  let vb = dot( gb, f - vec2f( 1.0, 0.0 ) );
  let vc = dot( gc, f - vec2f( 0.0, 1.0 ) );
  let vd = dot( gd, f - vec2f( 1.0, 1.0 ) );
  // Quintic fade ( Perlin's improved curve ) : zero first AND second
  // derivative at the cell borders, unlike the cubic smoothstep fade.
  let u = f * f * f * ( f * ( f * 6.0 - 15.0 ) + 10.0 );
  return mix( mix( va, vb, u.x ), mix( vc, vd, u.x ), u.y );
}
