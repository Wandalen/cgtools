#version 300 es

precision mediump float;

uniform vec4 u_color;
uniform float u_blue_offset;
out vec4 frag_color;

void main()
{
  frag_color = vec4( 0.0, 0.0, u_blue_offset, 0.0 ) + u_color;
}
