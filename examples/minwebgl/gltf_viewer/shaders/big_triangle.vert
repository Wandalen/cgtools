

out vec2 vUv;

void main()
{
  int x = gl_VertexID % 2;
  int y = gl_VertexID / 2;

  vUv = vec2( x, y ) * 2.0;
  gl_Position = vec4( vec2( x, y ) * 4.0 - vec2( 1.0 ), 0.0, 1.0 );
}