#version 300 es

layout ( location = 0 ) in vec3 a_position;

uniform mat4 u_mvp;

void main()
{
  // this shader is meant for drawing object's id into texture
  // vertex shader just transforms vertices to screen-space coordinates
  gl_Position = u_mvp * vec4( a_position, 1.0 );
}
