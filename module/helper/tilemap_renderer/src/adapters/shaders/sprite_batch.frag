#version 300 es
precision highp float;

in vec2 v_uv;
in vec4 v_tint;

uniform sampler2D u_texture;
// Coverage cut-off. Fragments whose sampled texture alpha (= coverage, since
// the alpha channel is the shape mask for both straight and premultiplied
// sheets) is below this are discarded — no colour AND no depth write, so an
// overlapping neighbour can still cover the pixel. 0.0 (default) keeps every
// fragment. Tested against tex.a BEFORE the tint multiply so a translucent
// tint (e.g. a 0.25-alpha drop shadow) doesn't pull every fragment under the
// threshold.
uniform float u_alpha_clip;

out vec4 frag_color;

void main()
{
  vec4 tex = texture( u_texture, v_uv );
  if ( tex.a < u_alpha_clip ) { discard; }
  frag_color = tex * v_tint;
}
