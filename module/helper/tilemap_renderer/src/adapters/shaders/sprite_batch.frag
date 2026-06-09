#version 300 es
precision highp float;

in vec2 v_uv;
in vec4 v_tint;

uniform sampler2D u_texture;

out vec4 frag_color;

void main()
{
  frag_color = texture( u_texture, v_uv ) * v_tint;
}
