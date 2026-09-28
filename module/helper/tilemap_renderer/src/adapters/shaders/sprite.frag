#version 300 es
precision highp float;

in vec2 v_uv;

uniform sampler2D u_texture;
uniform vec4 u_tint; // multiply with texture color
uniform bool u_premultiplied; // texture RGB is already scaled by its alpha

out vec4 frag_color;

void main()
{
  // A premultiplied texel must stay premultiplied after tinting, so the tint's
  // own alpha has to scale RGB as well — otherwise a faded sprite keeps its
  // full colour while its coverage drops (too bright under the ONE blend).
  vec4 tint = u_tint;
  if ( u_premultiplied ) { tint.rgb *= tint.a; }
  vec4 tex = texture( u_texture, v_uv );
  frag_color = tex * tint;
}
