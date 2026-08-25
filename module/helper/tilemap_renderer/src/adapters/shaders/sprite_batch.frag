#version 300 es
precision highp float;

in vec2 v_uv;
in vec4 v_tint;

uniform sampler2D u_texture;

out vec4 frag_color;

// Two variants are compiled from this one source (see `SpriteRenderer::new`):
//
//   • USE_ALPHA_CLIP defined  → the coverage-cutout variant. Fragments whose
//     sampled texture alpha (= coverage, since the alpha channel is the shape
//     mask for both straight and premultiplied sheets) fall below `u_alpha_clip`
//     are discarded — no colour AND no depth write, so an overlapping neighbour
//     can still cover the pixel, and the transparent quad corners of a
//     junction tile don't seal the depth buffer. Tested against tex.a BEFORE
//     the tint multiply so a translucent tint (e.g. a 0.25-alpha drop shadow)
//     doesn't pull every fragment under the threshold. Used by the opaque
//     cutout layers (`alpha_clip > 0`: terrain, region, selected, attack).
//
//   • USE_ALPHA_CLIP undefined → the plain variant, with NO `discard`. A
//     shader that can discard forces the GPU to disable early-Z (it can't
//     reject a fragment before running a shader that might kill it), so every
//     `alpha_clip == 0` layer (background, terrain_side, river, objects, fx,
//     preview) uses THIS variant. Discard-free ⇒ early-Z re-engages ⇒ these
//     full-board layers are depth-rejected under the opaque terrain BEFORE the
//     fragment shader runs, instead of shading then failing the late test.
#ifdef USE_ALPHA_CLIP
uniform float u_alpha_clip;
#endif

void main()
{
  vec4 tex = texture( u_texture, v_uv );
#ifdef USE_ALPHA_CLIP
  if ( tex.a < u_alpha_clip ) { discard; }
#endif
  frag_color = tex * v_tint;
}
