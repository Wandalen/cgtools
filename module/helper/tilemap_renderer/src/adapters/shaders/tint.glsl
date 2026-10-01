// Spliced into every fragment shader in place of its `#include "tint.glsl"`
// line, after the precision statement ( see `fragment_source` in webgl/webgl_renderers.rs ).

uniform bool u_premultiplied; // the bound texture's RGB is already scaled by its alpha

// The tinted texel, in the form the blend expects: straight, or premultiplied
// when `u_premultiplied` is set ( `blend_apply` then takes source factor ONE ).
//
// A premultiplied texel has to draw like its straight twin. The straight result
// is `tex * tint`, whose colour and alpha the RGBA8 target each clamps to 0..1
// before blending: colour `clamp( c * tint.rgb )`, alpha `clamp( a * tint.a )`.
// Scaling the stored `c * a` by the tint matches that only while both products
// stay in range, so a brightening tint or a tint alpha above 1 drew brighter
// than the straight twin. Instead the clamped colour is rebuilt from the stored
// texel: `clamp( c * a * tint.rgb, 0, a )` is `a * clamp( c * tint.rgb )`, and
// rescaling it from `a` to the clamped output alpha premultiplies it again.
vec4 tinted( vec4 tex, vec4 tint )
{
  if ( !u_premultiplied ) { return tex * tint; }
  float alpha = clamp( tex.a * tint.a, 0.0, 1.0 );
  if ( tex.a <= 0.0 ) { return vec4( 0.0 ); }
  vec3 rgb = clamp( tex.rgb * tint.rgb, vec3( 0.0 ), vec3( tex.a ) );
  return vec4( rgb * ( alpha / tex.a ), alpha );
}
