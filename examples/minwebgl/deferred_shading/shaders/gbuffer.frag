#version 300 es

precision mediump float;

in vec3 v_position;
in vec3 v_normal;
in vec2 v_tex_coord;

uniform sampler2D u_base_color;

layout( location = 0 ) out vec4 position;
layout( location = 1 ) out vec4 normal;
layout( location = 2 ) out vec4 color;

void main()
{
  position = vec4( v_position, 1.0 );
  normal = vec4( normalize( v_normal ), 1.0 );
  color = texture( u_base_color, v_tex_coord );
}
