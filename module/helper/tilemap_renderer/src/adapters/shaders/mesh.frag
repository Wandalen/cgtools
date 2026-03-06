#version 300 es
precision highp float;

in vec2 v_uv;
in vec2 v_pos;

uniform vec4 u_color;         // solid fill color
uniform sampler2D u_texture;  // optional texture
uniform bool u_use_texture;   // whether to sample texture

out vec4 frag_color;

void main()
{
  if ( u_use_texture )
  {
    vec4 tex = texture( u_texture, v_uv );
    frag_color = tex * u_color;
  }
  else
  {
    frag_color = u_color;
  }
}
