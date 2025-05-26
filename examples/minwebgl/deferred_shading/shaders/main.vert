#version 300 es

layout( location = 0 ) in vec3 a_position;
layout( location = 1 ) in vec3 a_normal;
layout( location = 2 ) in vec2 a_tex_coord;

uniform mat4 u_model;
uniform mat4 u_mvp;
uniform mat4 u_rotation;

out vec3 v_position;
out vec3 v_normal;
out vec2 v_tex_coord;

void main()
{
  v_position = ( u_model * vec4( a_position, 1.0 ) ).xyz;
  v_normal = ( u_rotation * vec4( a_normal, 1.0 ) ).xyz;
  v_tex_coord = a_tex_coord;
  gl_Position = u_mvp * vec4( a_position, 1.0 );
}
