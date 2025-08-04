#version 300 es

precision mediump float;

in vec3 v_position;
in vec3 v_normal;
in vec2 v_tex_coord;

uniform sampler2D u_base_color;

layout( location = 0 ) out vec4 position;
layout( location = 1 ) out vec4 normal;
layout( location = 2 ) out vec4 color;

vec3 SrgbToLinear( const in vec3 color )
{
  vec3 more = pow( color * 0.9478672986 + vec3( 0.0521327014 ), vec3( 2.4 ) );
  vec3 less = color * 0.0773993808;

  return mix( more, less, vec3( lessThanEqual( color, vec3( 0.04045 ) ) ) );
}

void main()
{
  // Just fill gbuffer
  position = vec4( v_position, 1.0 );
  normal = vec4( normalize( v_normal ), 1.0 );

  vec3 linear_color = SrgbToLinear( texture( u_base_color, v_tex_coord ).rgb );
  color = vec4 ( linear_color, 1.0 );
}
