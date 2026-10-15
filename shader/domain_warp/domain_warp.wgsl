//@ name: domain_warp
//@ description: Warps a 2D point by two centered fbm3 offsets for organic distortion.
//@ tags: category:noise, technique:warp
//@ depends_on: fbm3
//@ export: fn domain_warp(p: vec2f, strength: f32, lacunarity: f32, gain: f32, seed: f32) -> vec2f
//@ export: fn domain_warp_preview(p: vec2f, strength: f32, lacunarity: f32, gain: f32, seed: f32) -> f32
//@ param: strength argument f32 range(0.0, 1.5)
//@ param: lacunarity argument f32 range(1.0, 3.0)
//@ param: gain argument f32 range(0.0, 1.0)
//@ param: seed argument f32 range(-50.0, 50.0)

// lacunarity/gain forward straight through to both underlying fbm3 reads --
// this chunk has no octave structure of its own to tune. seed offsets the
// second read's decorrelation vector ( vec2f + f32 broadcasts per-component
// in WGSL ), reshuffling which warp pattern pairs with which base fbm3
// without touching the first read at all; seed = 0 ( this range's midpoint )
// reproduces the original, fixed ( 5.2, 1.3 ) offset exactly. fbm_max
// replaces the old hardcoded 0.875 -- fbm3's true output ceiling now
// depends on gain, and 0.875 was only ever that formula evaluated at
// gain = 0.5.
fn domain_warp( p : vec2f, strength : f32, lacunarity : f32, gain : f32, seed : f32 ) -> vec2f
{
  let fbm_max = 0.5 * ( 1.0 + gain + gain * gain );
  // Two decorrelated fbm3 reads ( the offset breaks the correlation ),
  // rescaled from fbm3's [0, fbm_max] range to a centered [-1, 1].
  let q = vec2f
  (
    fbm3( p, lacunarity, gain ),
    fbm3( p + vec2f( 5.2, 1.3 ) + seed, lacunarity, gain )
  );
  return p + strength * ( q * ( 2.0 / fbm_max ) - vec2f( 1.0 ) );
}

fn domain_warp_preview( p : vec2f, strength : f32, lacunarity : f32, gain : f32, seed : f32 ) -> f32
{
  let fbm_max = 0.5 * ( 1.0 + gain + gain * gain );
  return fbm3( domain_warp( p, strength, lacunarity, gain, seed ), lacunarity, gain ) / fbm_max;
}
