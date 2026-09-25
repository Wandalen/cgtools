#version 300 es
precision highp float;
// Texel addressing below; the fragment-stage default `mediump int` is 16-bit on Mali.
precision highp int;

uniform sampler2D sourceTexture;

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

void main()
{
  vec4 src = texture( sourceTexture, vUv );

  // Background pixels are cleared with alpha = 0 and must bypass tone mapping
  // ( as the clear color does in three.js ); geometry writes alpha = 1.
  if( src.a <= 0.0 ) { frag_color = vec4( src.rgb, 1.0 ); return; }
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
  ivec2 size = textureSize( sourceTexture, 0 );
  ivec2 p = ivec2( vUv * vec2( size ) );
  vec3 backgroundSum = vec3( 0.0 );
  float backgroundCount = 0.0;
  for( int y = -1; y <= 1; y++ )
  {
    for( int x = -1; x <= 1; x++ )
    {
      if( x == 0 && y == 0 ) { continue; }
      vec4 n = texelFetch( sourceTexture, clamp( p + ivec2( x, y ), ivec2( 0 ), size - 1 ), 0 );
      if( n.a <= 0.0 ) { backgroundSum += n.rgb; backgroundCount += 1.0; }
    }
  }

  // No pure-background neighbour (a partially covered pixel wedged between geometry):
  // keep the previous blend rather than guess.
  if( backgroundCount == 0.0 )
  {
    frag_color = vec4( mix( src.rgb, aces_tone_map( src.rgb ), src.a ), 1.0 );
    return;
  }

  vec3 background = backgroundSum / backgroundCount;
  // Clamped: the background is an estimate, and a small error divided by a small
  // coverage must not turn into negative light. A tiny `a` can inflate `geometry`,
  // but aces_tone_map() clamps to 1, so its contribution stays bounded by `a`.
  vec3 geometry = max( ( src.rgb - ( 1.0 - src.a ) * background ) / src.a, vec3( 0.0 ) );
  frag_color = vec4( src.a * aces_tone_map( geometry ) + ( 1.0 - src.a ) * background, 1.0 );
}
