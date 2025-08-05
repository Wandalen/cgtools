#version 300 es
precision highp float;

uniform vec3 u_color;
out vec4 frag_color;

void main()
{
  vec3 col = vec3( 112.21, 201.45, 94.35 ) / 255.0;
  frag_color = vec4( u_color, 1.0 );
}
