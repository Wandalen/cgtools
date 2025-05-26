#version 300 es

precision mediump float;

in vec2 v_tex_coord;

uniform sampler2D u_colors;

layout( location = 0 ) out vec4 frag_color;

void main()
{
  frag_color = texture( u_colors, v_tex_coord );
}
