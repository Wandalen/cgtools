#version 300 es

layout ( location = 0 ) in vec3 a_position;
layout ( location = 1 ) in vec3 a_normal;

uniform mat4 u_mvp;

void main()
{
  // this shader extrudes positions in direction of normals
  
  vec4 clip = u_mvp * vec4( a_position, 1.0 );
  // multiplying by clip.z makes outline size independent of distance
  vec3 pos = a_position + a_normal * clip.z * 0.005;
  gl_Position = u_mvp * vec4( pos, 1.0 );
}
