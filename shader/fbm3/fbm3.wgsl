//@ name: fbm3
//@ description: Fixed 3-octave fractal Brownian motion built on value_noise, in [0, 0.5*(1+gain+gain^2)].
//@ tags: category:noise, technique:fractal
//@ depends_on: value_noise
//@ export: fn fbm3(p: vec2f, lacunarity: f32, gain: f32) -> f32
//@ param: lacunarity argument f32 range(1.0, 3.0)
//@ param: gain argument f32 range(0.0, 1.0)

// lacunarity ( per-octave frequency multiplier ) and gain ( per-octave
// amplitude multiplier ) are the two classic FBM tunables. Octave count
// stays fixed at 3 -- that's this chunk's own identity ( its name ), not a
// hardcoded constant standing in for a missing parameter; a different
// octave count needs a differently-named chunk. Both default to their
// range's midpoint, reproducing this chunk's original 2.0/0.5 look and its
// original [0, 0.875] output range exactly.
fn fbm3( p_in : vec2f, lacunarity : f32, gain : f32 ) -> f32
{
  var p = p_in;
  var amplitude = 0.5;
  var value = 0.0;
  value += amplitude * value_noise( p, 0.0 );
  p *= lacunarity;
  amplitude *= gain;
  value += amplitude * value_noise( p, 0.0 );
  p *= lacunarity;
  amplitude *= gain;
  value += amplitude * value_noise( p, 0.0 );
  return value;
}
