#version 300 es

layout( location = 0 ) in vec3 a_position;
layout( location = 1 ) in vec3 a_translation;

uniform mat4 u_mvp;

void main()
{
  gl_Position = u_mvp * vec4( a_position + a_translation, 1.0 );
}
