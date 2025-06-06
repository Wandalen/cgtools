#version 300 es
out vec2 v_tex_coord;

void main()
{
  float x = float( gl_VertexID / 2 );
  float y = float( gl_VertexID % 2 );

  v_tex_coord = vec2( x, y ) * 2.0;
  gl_Position = vec4( vec2( x * 4.0 - 1.0, y * 4.0 - 1.0  ), 0.0, 1.0 );
}