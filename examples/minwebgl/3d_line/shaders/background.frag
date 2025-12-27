#version 300 es
precision highp float;

out vec4 frag_color;

void main()
{
  vec3 col = vec3( 1.0 );
  frag_color = vec4( col , 1.0 );
}
