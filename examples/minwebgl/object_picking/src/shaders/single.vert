#version 300 es

layout ( location = 0 ) in vec3 a_position;
layout ( location = 1 ) in vec3 a_normal;
layout ( location = 2 ) in vec2 a_texcoord;

uniform mat4 u_model;
uniform mat4 u_projection_view;
uniform mat3 u_norm_mat;
uniform int u_id;

out vec3 v_normal;
out vec3 v_frag_pos;
out vec2 v_texcoord;
flat out int v_id;

void main()
{
  gl_Position = u_projection_view * u_model * vec4( a_position, 1.0 );
  v_normal = u_norm_mat * a_normal;
  v_frag_pos = ( u_model * vec4( a_position, 1.0 ) ).xyz;
  v_texcoord = a_texcoord;
  v_id = u_id;
}
