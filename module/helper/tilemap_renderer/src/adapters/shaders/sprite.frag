#version 300 es
precision highp float;

in vec2 v_uv;
in vec2 v_pos;

uniform sampler2D u_texture;
uniform vec4 u_tint; // multiply with texture color

out vec4 frag_color;

void main()
{
  vec4 tex = texture( u_texture, v_uv );
  frag_color = tex * u_tint;
}
