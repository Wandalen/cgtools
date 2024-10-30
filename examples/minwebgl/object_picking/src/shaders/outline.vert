#version 300 es

layout ( location=0 ) in vec3 a_position;
layout ( location=1 ) in vec3 a_normal;

uniform mat4 u_mvp;

void main()
{
  vec3 position = ( a_position + a_normal * 0.7 );
  gl_Position = u_mvp * vec4( position, 1.0 );
}
