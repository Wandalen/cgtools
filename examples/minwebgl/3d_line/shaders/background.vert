#version 300 es
precision highp float;

void main() 
{
  float x = float( gl_VertexID % 2 );
  float y = float( gl_VertexID / 2 );

  gl_Position = vec4( x * 4.0 - 1.0, 1.0 - y * 4.0, 1.0, 1.0 );
}