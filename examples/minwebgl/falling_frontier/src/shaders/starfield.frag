#version 300 es
precision highp float;

in vec3 v_color;
in float v_alpha;

out vec4 frag_color;

void main()
{
  // Per-star alpha (see starfield.rs's MIN_ALPHA/MAX_ALPHA), not a flat
  // value - with additive blending (see starfield.rs's draw()) alpha
  // directly controls how much each star brightens the sky behind it, so
  // randomizing it (alongside point size) is what makes stars read as
  // individually distinct twinkles instead of one uniform dust haze.
  frag_color = vec4( v_color, v_alpha );
}
