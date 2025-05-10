#version 300 es

precision mediump float;

uniform sampler2D positions;
uniform sampler2D normals;

layout( location = 0 ) out vec4 frag_color;

void main()
{
  frag_color = vec4( 1.0, 1.0, 0.0, 1.0 );
}
