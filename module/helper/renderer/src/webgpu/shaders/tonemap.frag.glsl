#version 300 es

// GLSL 300 es twin of `tonemap.wgsl` `fs_main`, consumed by the gpu_hal WebGL2
// backend: ACES tone map + sRGB encode with the alpha-0 background bypass.
// `tonemap.wgsl` is the canonical source ( ADR-001 §5 ) — edit it first and
// mirror changes here. The sampler uniform is named after its HAL texture
// entry `tex_{group}_{binding}`.

precision highp float;

// HDR source ( rgba16f ); read by texel, no filtering.
uniform highp sampler2D tex_0_0;

layout( location = 0 ) out vec4 frag_color;

vec3 aces_tone_map( vec3 hdr )
{
  mat3 m1 = mat3
  (
    vec3( 0.59719, 0.07600, 0.02840 ),
    vec3( 0.35458, 0.90834, 0.13383 ),
    vec3( 0.04823, 0.01566, 0.83777 )
  );
  mat3 m2 = mat3
  (
    vec3( 1.60475, -0.10208, -0.00327 ),
    vec3( -0.53108, 1.10813, -0.07276 ),
    vec3( -0.07367, -0.00605, 1.07602 )
  );

  // Pre-exposure RRT scaling, matching three.js ACESFilmicToneMapping.
  vec3 v = m1 * ( hdr / 0.6 );
  vec3 a = v * ( v + 0.0245786 ) - 0.000090537;
  vec3 b = v * ( 0.983729 * v + 0.4329510 ) + 0.238081;

  return clamp( m2 * ( a / b ), vec3( 0.0 ), vec3( 1.0 ) );
}

vec3 linear_to_srgb( vec3 color )
{
  vec3 more = pow( color, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 );
  vec3 less = color * 12.92;

  return mix( more, less, lessThanEqual( color, vec3( 0.0031308 ) ) );
}

void main()
{
  vec4 src = texelFetch( tex_0_0, ivec2( gl_FragCoord.xy ), 0 );

  // Background pixels are cleared with alpha = 0 and bypass tone mapping
  // ( as the clear color does in three.js ); geometry writes alpha = 1.
  vec3 mapped = mix( src.rgb, aces_tone_map( src.rgb ), src.a );

  frag_color = vec4( linear_to_srgb( mapped ), 1.0 );
}
