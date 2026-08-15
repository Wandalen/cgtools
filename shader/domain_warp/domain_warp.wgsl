//@ name: domain_warp
//@ description: Warps a 2D point by two centered fbm3 offsets for organic distortion.
//@ tags: category:noise, technique:warp
//@ depends_on: fbm3
//@ export: fn domain_warp(p: vec2f, strength: f32) -> vec2f
//@ export: fn domain_warp_preview(p: vec2f) -> f32

fn domain_warp( p : vec2f, strength : f32 ) -> vec2f
{
  // Two decorrelated fbm3 reads ( the offset breaks the correlation ),
  // rescaled from fbm3's [0, 0.875] range to a centered [-1, 1].
  let q = vec2f
  (
    fbm3( p ),
    fbm3( p + vec2f( 5.2, 1.3 ) )
  );
  return p + strength * ( q * ( 2.0 / 0.875 ) - vec2f( 1.0 ) );
}

fn domain_warp_preview( p : vec2f ) -> f32
{
  return fbm3( domain_warp( p, 0.6 ) ) / 0.875;
}
