#version 300

precision mediump float;

in vec2 v_tex_coord;

uniform sampler2D u_image;

layout( location = 0 ) out vec4 frag_color;

void main()
{
  frag_color = texture( u_image, v_tex_coord );
}
