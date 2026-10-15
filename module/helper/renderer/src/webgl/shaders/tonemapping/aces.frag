#version 300 es
precision highp float;

uniform sampler2D sourceTexture;
// The renderer's WBOIT accumulation target ( `Renderer::transparent_coverage_texture` ):
// its alpha is the revealage of the transparent layers, 1 where none covers the pixel.
// Bound only when `useTransparentCoverage` is set; without it the MSAA-edge
// reconstruction below stays off.
uniform sampler2D transparentCoverage;
uniform bool useTransparentCoverage;

in vec2 vUv;
out vec4 frag_color;

vec3 aces_tone_map( vec3 hdr )
{
  mat3x3 m1 = mat3x3
  (
    0.59719, 0.07600, 0.02840,
    0.35458, 0.90834, 0.13383,
    0.04823, 0.01566, 0.83777
  );
  mat3x3 m2 = mat3x3
  (
    1.60475, -0.10208, -0.00327,
    -0.53108,  1.10813, -0.07276,
    -0.07367, -0.00605,  1.07602
  );

  // Pre-exposure RRT scaling, matching three.js ACESFilmicToneMapping.
  vec3 v = m1 * ( hdr / 0.6 );
  vec3 a = v * ( v + 0.0245786 ) - 0.000090537;
  vec3 b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;

  return clamp( m2 * ( a / b ), vec3( 0.0 ), vec3( 1.0 ) );
}

// The plain blend for a partially covered pixel: no background estimate needed.
vec3 coverage_blend( vec4 src )
{
  return mix( src.rgb, aces_tone_map( src.rgb ), src.a );
}

// Averages the pure-background ( alpha = 0 ) pixels among the 8 neighbours of `p` into
// `background` and returns how many there were. Neighbours outside the image are
// skipped rather than clamped onto the edge, which would count one pixel several times.
float background_estimate( ivec2 p, ivec2 size, out vec3 background )
{
  vec3 sum = vec3( 0.0 );
  float count = 0.0;
  for( int y = -1; y <= 1; y++ )
  {
    for( int x = -1; x <= 1; x++ )
    {
      ivec2 q = p + ivec2( x, y );
      if( ( x == 0 && y == 0 ) || any( lessThan( q, ivec2( 0 ) ) ) || any( greaterThanEqual( q, size ) ) ) { continue; }
      vec4 n = texelFetch( sourceTexture, q, 0 );
      if( n.a <= 0.0 ) { sum += n.rgb; count += 1.0; }
    }
  }
  background = count > 0.0 ? sum / count : vec3( 0.0 );
  return count;
}

void main()
{
  vec4 src = texture( sourceTexture, vUv );

  // Background pixels are cleared with alpha = 0 and must bypass tone mapping
  // ( as the clear color does in three.js ); geometry writes alpha = 1.
  // The coverage alpha is forwarded (not hardcoded to 1) so a caller can composite
  // the canvas over other content (e.g. AR over a photo) with real per-pixel coverage.
  if( src.a <= 0.0 ) { frag_color = vec4( src.rgb, src.a ); return; }
  if( src.a >= 1.0 ) { frag_color = vec4( aces_tone_map( src.rgb ), 1.0 ); return; }

  // Partially covered: an MSAA-resolved silhouette pixel holds the coverage-weighted
  // average of its samples, src.rgb = a * geometry + ( 1 - a ) * background, with
  // src.a = a. Tone mapping that mixture — or mix( src.rgb, mapped, a ), which also lets
  // (1 - a) of the raw, un-tone-mapped geometry HDR through — leaves a light fringe on
  // bright edges. The intended result is a * aces( geometry ) + ( 1 - a ) * background.
  // WebGL2 has no multisample textures, so samples can't be tone mapped before the
  // resolve; instead `background` is estimated from the pure-background (alpha = 0)
  // neighbours — which a silhouette pixel always has on its outer side — and
  // `geometry` is solved for. That covers the flat clear colour and the shadowed
  // ground alike. Neighbours are addressed in the source's own texel grid.
  //
  // WBOIT transparency over the background also leaves 0 < a < 1 ( a = the transparent
  // coverage ), and only the pixels on a transparent mesh's outline have an alpha-0
  // neighbour. Reconstructing those but not the interior ones would ring every glass
  // surface seen against the background with a 1-px outline of a different brightness,
  // so any pixel with transparent coverage takes the plain blend, edge and interior
  // alike. Without the coverage texture the pass can't tell glass from a silhouette, so
  // every partially covered pixel takes the plain blend.
  ivec2 size = textureSize( sourceTexture, 0 );
  ivec2 p = ivec2( vUv * vec2( size ) );
  if( !useTransparentCoverage || texelFetch( transparentCoverage, p, 0 ).a < 1.0 )
  {
    frag_color = vec4( coverage_blend( src ), src.a );
    return;
  }

  // No pure-background neighbour, so no background estimate: every neighbour is
  // geometry ( a pixel wedged between two surfaces ) or covered by transparency. Fall
  // back to the plain blend rather than guess.
  vec3 background;
  if( background_estimate( p, size, background ) == 0.0 )
  {
    frag_color = vec4( coverage_blend( src ), src.a );
    return;
  }

  // Clamped: the background is an estimate, and a small error divided by a small
  // coverage must not turn into negative light. A tiny `a` can inflate `geometry`,
  // but aces_tone_map() clamps to 1, so its contribution stays bounded by `a`.
  vec3 geometry = max( ( src.rgb - ( 1.0 - src.a ) * background ) / src.a, vec3( 0.0 ) );
  frag_color = vec4( src.a * aces_tone_map( geometry ) + ( 1.0 - src.a ) * background, src.a );
}
