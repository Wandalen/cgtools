#version 300 es
precision mediump float;

layout( location = 0 ) out vec4 frag_color;

void main()
{
  // shader for drawing outline of an object
  frag_color = vec4(0.1f, 0.8f, 0.1f, 1.0f);
}
