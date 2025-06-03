#version 300 es

precision mediump float;

in vec2 v_tex_coord;

uniform sampler2D u_colors;

layout( location = 0 ) out vec4 frag_color;

vec3 LinearToSrgb( const in vec3 color )
{
  vec3 more = pow( color, vec3( 0.41666 ) ) * 1.055 - vec3( 0.055 );
  vec3 less = color * 12.92;

  return mix( more, less, vec3( lessThanEqual( color, vec3( 0.0031308 ) ) ) );
}

void main()
{
  frag_color = vec4( LinearToSrgb( texture( u_colors, v_tex_coord ).rgb ), 1.0 );
}
