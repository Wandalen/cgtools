#version 300 es

// Same "big triangle" trick as `renderer`'s own `VS_TRIANGLE` (one
// attribute-less triangle covering the whole viewport via `gl_VertexID`),
// re-derived locally so the varying is raw NDC instead of a 0..2 UV -
// `background.frag` needs NDC to reconstruct a per-pixel world-space ray.
out vec2 v_ndc;

void main()
{
  float x = float( gl_VertexID / 2 );
  float y = float( gl_VertexID % 2 );

  vec2 pos = vec2( x * 4.0 - 1.0, y * 4.0 - 1.0 );
  v_ndc = pos;
  gl_Position = vec4( pos, 0.0, 1.0 );
}
