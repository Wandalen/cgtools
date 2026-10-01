#version 300 es
precision highp float;

in vec2 v_uv;

uniform sampler2D u_texture;
uniform vec4 u_tint; // multiply with texture color

#include "tint.glsl"

out vec4 frag_color;

void main()
{
  frag_color = tinted( texture( u_texture, v_uv ), u_tint );
}
