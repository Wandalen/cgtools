//@ name: gradient_noise
//@ description: Quintic-faded gradient (Perlin) noise at a 2D point, roughly in [-0.7, 0.7].
//@ tags: category:noise, technique:gradient
//@ depends_on: hash22
//@ export: fn gradient_noise(p: vec2f, seed: f32) -> f32
//@ param: seed argument f32 range(-50.0, 50.0)

// seed offsets the integer lattice coordinate fed into each corner's hash,
// not p itself -- panning p by an integer amount just relabels the same
// corner -> gradient mapping, whereas offsetting the coordinate that's
// actually hashed reshuffles it, since hash22 has no smoothness to
// preserve ( same technique/justification as voronoi's seed param ).
// seed = 0 ( this range's midpoint ) reproduces the original, unseeded
// pattern exactly.
fn gradient_noise( p : vec2f, seed : f32 ) -> f32
{
  let i = floor( p );
  let f = fract( p );
  // Per-corner pseudo-random gradients in [-1, 1]^2, dotted with the
  // offset from that corner to the sample point.
  let ga = hash22( i + seed ) * 2.0 - vec2f( 1.0 );
  let gb = hash22( i + vec2f( 1.0, 0.0 ) + seed ) * 2.0 - vec2f( 1.0 );
  let gc = hash22( i + vec2f( 0.0, 1.0 ) + seed ) * 2.0 - vec2f( 1.0 );
  let gd = hash22( i + vec2f( 1.0, 1.0 ) + seed ) * 2.0 - vec2f( 1.0 );
  let va = dot( ga, f );
  let vb = dot( gb, f - vec2f( 1.0, 0.0 ) );
  let vc = dot( gc, f - vec2f( 0.0, 1.0 ) );
  let vd = dot( gd, f - vec2f( 1.0, 1.0 ) );
  // Quintic fade ( Perlin's improved curve ) : zero first AND second
  // derivative at the cell borders, unlike the cubic smoothstep fade.
  let u = f * f * f * ( f * ( f * 6.0 - 15.0 ) + 10.0 );
  return mix( mix( va, vb, u.x ), mix( vc, vd, u.x ), u.y );
}
