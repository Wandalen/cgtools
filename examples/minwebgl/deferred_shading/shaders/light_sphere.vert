#version 300 es

layout( location = 0 ) in vec3 a_position;
layout( location = 1 ) in vec3 a_translation;

uniform mat4 u_view_projection;
uniform float u_scale;

void main()
{
  // Scale the sphere and translate to light position
  vec3 world_pos = a_position * u_scale + a_translation;
  gl_Position = u_view_projection * vec4( world_pos, 1.0 );
}
