#version 300 es
precision mediump float;

layout ( location = 0 ) out vec4 frag_color;

uniform vec3 u_color;

void main()
{
  frag_color = vec4( u_color, 1.0 );
}
