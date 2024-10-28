#version 300 es
precision mediump float;

in vec2 v_tex_coord;
out vec4 frag_color;

uniform sampler2D u_image;
uniform vec2 u_texel_size;

vec3 apply_3x3_kernel( float kernel[ 9 ] )
{
  const vec2 OFFSETS[] = vec2[]
  (
    vec2( -1.0,  1.0 ), vec2( 0.0,  1.0 ), vec2( 1.0,  1.0 ),
    vec2( -1.0,  0.0 ), vec2( 0.0,  0.0 ), vec2( 1.0,  0.0 ),
    vec2( -1.0, -1.0 ), vec2( 0.0, -1.0 ), vec2( 1.0, -1.0 )
  );

  vec3 sum = vec3( 0.0 );
  for( int i = 0; i < 9; i++ )
  {
    vec2 offset = v_tex_coord + OFFSETS[ i ] * u_texel_size;
    vec4 pixel = texture( u_image, offset );
    sum += pixel.rgb * kernel[ i ];
  }

  return sum;
}

void main()
{
  const float EMBOSS_KERNEL[ 9 ] = float[]
  (
    -2.0, -1.0, 0.0,
    -1.0,  1.0, 1.0,
     0.0,  1.0, 2.0
  );

  frag_color = vec4( apply_3x3_kernel( EMBOSS_KERNEL ), 1.0 );
}
