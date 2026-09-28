#version 300 es
precision highp float;

in vec2 v_uv;
in vec4 v_tint;

uniform vec4 u_color;         // MeshBatchParams.fill — batch-level solid color
uniform sampler2D u_texture;  // optional texture
uniform bool u_use_texture;   // whether to sample texture
uniform bool u_premultiplied; // sampled texture RGB is already scaled by its alpha

out vec4 frag_color;

void main()
{
  // Per-instance tint (v_tint) modulates the batch-level fill and any sampled
  // texture. Passing tint = (1, 1, 1, 1) yields the same output as the
  // single-draw path (mesh.frag), which has no per-instance tint.
  if ( u_use_texture )
  {
    // Keep a premultiplied texel premultiplied after tinting (see sprite.frag).
    vec4 color = u_color * v_tint;
    if ( u_premultiplied ) { color.rgb *= color.a; }
    vec4 tex = texture( u_texture, v_uv );
    frag_color = tex * color;
  }
  else
  {
    frag_color = u_color * v_tint;
  }
}
