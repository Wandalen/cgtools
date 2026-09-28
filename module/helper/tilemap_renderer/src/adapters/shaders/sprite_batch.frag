#version 300 es
precision highp float;

in vec2 v_uv;
in vec4 v_tint;

uniform sampler2D u_texture;
uniform bool u_premultiplied; // texture RGB is already scaled by its alpha

out vec4 frag_color;

void main()
{
  // Keep a premultiplied texel premultiplied after tinting (see sprite.frag).
  vec4 tint = v_tint;
  if ( u_premultiplied ) { tint.rgb *= tint.a; }
  frag_color = texture( u_texture, v_uv ) * tint;
}
