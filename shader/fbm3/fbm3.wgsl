//@ name: fbm3
//@ description: Fixed 3-octave fractal Brownian motion built on value_noise, in [0, 0.875].
//@ tags: category:noise, technique:fractal
//@ depends_on: value_noise
//@ export: fn fbm3(p: vec2f) -> f32

fn fbm3( p_in : vec2f ) -> f32
{
  var p = p_in;
  var value = 0.0;
  value += 0.5 * value_noise( p );
  p *= 2.0;
  value += 0.25 * value_noise( p );
  p *= 2.0;
  value += 0.125 * value_noise( p );
  return value;
}
