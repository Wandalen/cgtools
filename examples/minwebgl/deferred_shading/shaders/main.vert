#version 300 es

layout( location = 0 ) in vec3 a_position;
layout( location = 1 ) in vec3 a_normal;
layout( location = 2 ) in vec2 a_tex_coord;

uniform mat4 u_model;
uniform mat4 u_rotation;
uniform mat4 u_mvp;

out vec3 v_position;
out vec3 v_normal;
out vec2 v_tex_coord;

void main()
{
  // Default vertex shader with normal and tex_coord attributes
  v_position = ( u_model * vec4( a_position, 1.0 ) ).xyz;
  v_normal = mat3( u_rotation ) * a_normal;
  v_tex_coord = a_tex_coord;
  gl_Position = u_mvp * vec4( a_position, 1.0 );
}
