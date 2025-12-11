#version 300 es

precision mediump float;

uniform mat4x4 viewMatrix;

out vec2 vUvs;

void main() 
{	
  float x = float( gl_VertexID / 2 );
  float y = float( gl_VertexID % 2 );

  vUvs = vec2( x * 2.0, y * 2.0 );

  gl_Position = vec4( -1.0 + x * 4.0, -1.0 + y * 4.0, 1.0, 1.0 );
}
