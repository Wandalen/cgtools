#version 300 es

out vec2 v_uv;

void main()
{
  // Big-triangle trick: 3 vertices, no buffer, gl_VertexID picks the corner.
  // The triangle overshoots clip space; only the visible unit square of v_uv
  // ( bottom-left = (0,0), top-right = (1,1) ) is ever rasterized to pixels.
  int x = gl_VertexID & 1;
  int y = gl_VertexID / 2;
  vec2 uv = vec2( float( x ) * 2.0, float( y ) * 2.0 );

  v_uv = uv;
  gl_Position = vec4( uv * 2.0 - 1.0, 0.0, 1.0 );
}
