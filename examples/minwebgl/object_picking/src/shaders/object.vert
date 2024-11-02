#version 300 es

layout ( location = 0 ) in vec3 a_position;
layout ( location = 1 ) in vec3 a_normal;
layout ( location = 2 ) in vec2 a_texcoord;

uniform mat4 u_model;
uniform mat4 u_projection_view;
uniform mat3 u_norm_mat;

out vec3 v_normal;
out vec3 v_frag_worldpos;
out vec2 v_texcoord;

void main()
{
  v_normal = u_norm_mat * a_normal;
  v_frag_worldpos = ( u_model * vec4( a_position, 1.0 ) ).xyz;
  v_texcoord = a_texcoord;
  gl_Position = u_projection_view * u_model * vec4( a_position, 1.0 );
}
