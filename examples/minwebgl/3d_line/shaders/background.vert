#version 300 es
precision highp float;

out vec2 vUv;

void main() 
{
  float x = float( gl_VertexID % 2 );
  float y = float( gl_VertexID / 2 );

  vUv = vec2( 2.0 * x, 1.0 - 2.0 * y );
  gl_Position = vec4( x * 4.0 - 1.0, 1.0 - y * 4.0, 1.0, 1.0 );
}