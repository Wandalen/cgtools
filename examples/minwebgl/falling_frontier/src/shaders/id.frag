#version 300 es
precision mediump float;

uniform int u_id;

layout( location = 0 ) out int frag_id;

void main()
{
  frag_id = u_id;
}
