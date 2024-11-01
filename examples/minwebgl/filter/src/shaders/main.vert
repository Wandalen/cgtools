#version 300 es

out vec2 v_tex_coord;

void main()
{
  int x = gl_VertexID & 1;
  int y = gl_VertexID / 2;
  vec2 tc = vec2( float( x ) * 2.0, float( y ) * 2.0 );

  v_tex_coord = tc;
  gl_Position = vec4
  (
    tc.x * 2.0 - 1.0,
    tc.y * 2.0 - 1.0,
    0.0,
    1.0
  );
}
