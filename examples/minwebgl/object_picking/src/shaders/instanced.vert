#version 300 es

layout ( location=0 ) in vec3 a_position;
layout ( location=1 ) in vec3 a_normal;
layout ( location=2 ) in vec2 a_texcoord;
layout ( location=3 ) in mat4 a_model;
layout ( location=7 ) in mat3 a_norm_mat;
layout ( location=10 ) in int a_id;

out vec3 v_normal;
out vec3 v_frag_pos;
out vec2 v_texcoord;
flat out int v_id;

uniform mat4 u_projection_view;

void main()
{
  gl_Position = u_projection_view * a_model * vec4( a_position, 1.0 );
  v_normal = a_norm_mat * a_normal;
  v_frag_pos = ( a_model * vec4( a_position, 1.0 ) ).xyz;
  v_texcoord = a_texcoord;
  v_id = a_id;
}
