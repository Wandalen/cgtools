#version 300 es
precision mediump float;

uniform vec3 u_color;
uniform float u_alpha;

out vec4 frag_color;

void main()
{
  frag_color = vec4( u_color, u_alpha );
}
