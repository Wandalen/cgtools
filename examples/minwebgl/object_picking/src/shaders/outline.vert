#version 300 es

layout ( location = 0 ) in vec3 a_position;
layout ( location = 1 ) in vec3 a_normal;

uniform mat4 u_mvp;

void main()
{
  // make outline size independent of distance
  vec4 clip = u_mvp * vec4( a_position, 1.0 );
  vec3 pos = a_position + a_normal * clip.z * 0.005;
  gl_Position = u_mvp * vec4( pos, 1.0 );
}
