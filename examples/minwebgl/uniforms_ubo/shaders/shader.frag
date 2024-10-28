#version 300 es
precision mediump float;

layout( std140 ) uniform ColorBlock
{
  vec4 u_color;
  float u_blue_offset;
};

out vec4 frag_color;

void main()
{
  frag_color = vec4( 0.0, 0.0, u_blue_offset, 0.0 ) + u_color;
}
