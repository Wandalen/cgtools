#version 300 es

precision mediump float;

in vec2 v_texcoord;
out vec4 frag_color;

uniform sampler2D u_tex_image;

void main()
{
  frag_color = texture( u_tex_image, v_texcoord );
}
