#version 300 es
precision highp float;

in vec2 v_uv;
in vec4 v_tint;

uniform vec4 u_color;         // MeshBatchParams.fill — batch-level solid color
uniform sampler2D u_texture;  // optional texture
uniform bool u_use_texture;   // whether to sample texture

out vec4 frag_color;

void main()
{
  // Per-instance tint (v_tint) modulates the batch-level fill and any sampled
  // texture. With tint = (1, 1, 1, 1) the output matches the single-draw mesh
  // path, so existing calls stay visually identical after the tint field is
  // added.
  if ( u_use_texture )
  {
    vec4 tex = texture( u_texture, v_uv );
    frag_color = tex * u_color * v_tint;
  }
  else
  {
    frag_color = u_color * v_tint;
  }
}
