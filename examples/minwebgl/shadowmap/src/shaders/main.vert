#version 300 es

layout( location = 0 ) in vec3 a_pos;
layout( location = 1 ) in vec3 a_normal;
layout( location = 2 ) in vec2 a_texcoord;

uniform mat4 u_mvp;
uniform mat4 u_model;

out vec3 v_world_pos;
out vec3 v_normal;
out vec2 v_texcoord;

void main()
{
  vec4 world_pos = u_model * vec4( a_pos, 1.0 );
  v_world_pos = world_pos.xyz;
  v_normal = mat3( u_model ) * a_normal;
  v_texcoord = a_texcoord;

  gl_Position = u_mvp * vec4( a_pos, 1.0 );
}
