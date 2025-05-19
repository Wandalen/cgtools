#version 300 es

out vec2 vUv;

void main()
{
  int x = gl_VertexID / 2;
  int y = gl_VertexID % 2;

  vUv = vec2( x, y ) * 2.0;
  vec4 clipPos = vec4( vec2( float( x ) * 4.0 - 1.0, 1.0 - float( y ) * 4.0 ), 0.0, 1.0 );

  gl_Position = clipPos;
}