#version 300 es

in vec2 a_position;

uniform vec2 u_offset;
uniform float u_scale;

void main()
{
  gl_Position = vec4( a_position * u_scale + u_offset, 0.0, 1.0 );
}
