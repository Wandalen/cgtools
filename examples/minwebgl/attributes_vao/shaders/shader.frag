#version 300 es

precision mediump float;

in vec4 v_color;
out vec4 frag_color;

void main()
{
  frag_color = v_color;
}
