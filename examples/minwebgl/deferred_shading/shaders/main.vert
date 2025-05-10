#version 300 es

layout( location = 0 ) in vec3 position;
layout( location = 1 ) in vec3 normal;

uniform mat4 u_model;
uniform mat4 u_mvp;

out vec3 v_position;
out vec3 v_normal;

void main()
{
  v_position = vec3( u_model * vec4( position, 1.0 ) );
  v_normal = normal;
  gl_Position = u_mvp * vec4( position, 1.0 );
}
