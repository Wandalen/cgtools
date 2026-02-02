#version 300 es

precision mediump float;

in vec2 v_tex_coord;

uniform sampler2D u_image;

layout( location = 0 ) out vec4 frag_color;

void main()
{
  vec4 color = texture( u_image, v_tex_coord );
  color.rgb *= color.a;
  frag_color = color;
}
