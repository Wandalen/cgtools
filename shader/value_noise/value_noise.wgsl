//@ name: value_noise
//@ description: Bilinear-interpolated value noise sampled at a 2D point, in [0, 1).
//@ tags: category:noise
//@ depends_on: hash21
//@ export: fn value_noise(p: vec2f, seed: f32) -> f32
//@ param: seed argument f32 range(-50.0, 50.0)

// seed offsets the integer lattice coordinate fed into each corner's hash
// -- same technique/justification as voronoi's and gradient_noise's seed.
// seed = 0 ( this range's midpoint ) reproduces the original, unseeded
// pattern exactly.
fn value_noise( p : vec2f, seed : f32 ) -> f32
{
  let i = floor( p );
  let f = fract( p );
  let a = hash21( i + seed );
  let b = hash21( i + vec2f( 1.0, 0.0 ) + seed );
  let c = hash21( i + vec2f( 0.0, 1.0 ) + seed );
  let d = hash21( i + vec2f( 1.0, 1.0 ) + seed );
  let u = f * f * ( 3.0 - 2.0 * f );
  return mix( mix( a, b, u.x ), mix( c, d, u.x ), u.y );
}
