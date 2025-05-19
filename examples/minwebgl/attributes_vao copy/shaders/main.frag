#version 300 es
precision mediump float;

uniform samplerCube env_map;
uniform sampler2D lut;

out vec4 frag_color;
in vec3 worldDir;
in vec2 vUv;

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

  vec3 v = m1 * hdr;
  vec3 a = v * ( v + 0.0245786 ) - 0.000090537;
  vec3 b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;

  return clamp( m2 * ( a / b ), vec3( 0.0 ), vec3( 1.0 ) );
}

void main()
{
  
  vec3 color = aces_tone_map( texture( env_map, worldDir ).xyz );
  //color = texture( lut, vUv ).xyz * 5.0;
  //color = texture( env_map, worldDir ).xyz * 1000.0;
  frag_color = vec4( color, 1.0 );
}