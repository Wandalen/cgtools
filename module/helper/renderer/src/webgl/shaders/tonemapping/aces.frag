#version 300 es
precision highp float;

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
  vec3 mapped = aces_tone_map( src.rgb );
  // Background pixels are cleared with alpha = 0 and must bypass tone mapping
  // ( as the clear color does in three.js ); geometry writes alpha = 1.
  frag_color = vec4( mix( src.rgb, mapped, src.a ), 1.0 );
}
