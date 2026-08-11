//@ name: value_noise
//@ description: Bilinear-interpolated value noise sampled at a 2D point, in [0, 1).
//@ depends_on: hash21
//@ export: fn value_noise(p: vec2f) -> f32

fn value_noise( p : vec2f ) -> f32
{
  let i = floor( p );
  let f = fract( p );
  let a = hash21( i );
  let b = hash21( i + vec2f( 1.0, 0.0 ) );
  let c = hash21( i + vec2f( 0.0, 1.0 ) );
  let d = hash21( i + vec2f( 1.0, 1.0 ) );
  let u = f * f * ( 3.0 - 2.0 * f );
  return mix( mix( a, b, u.x ), mix( c, d, u.x ), u.y );
}
