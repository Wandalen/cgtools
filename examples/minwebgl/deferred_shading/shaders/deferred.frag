#version 300 es

precision mediump float;

layout( location = 0 ) out vec4 frag_color;

void main()
{
  vec4 frag_coord = gl_FragCoord;
  frag_color = vec4( 1.0, 1.0, 0.0, 1.0 );
}
