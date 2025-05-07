#version 300 es 

uniform mat4x4 viewMatrix;
uniform mat4x4 projectionMatrix;

const vec2 positions[ 4 ] = vec2[]
(
  vec2( -1.0, 1.0 ),    // Top-left vertex
  vec2( 1.0, 1.0 ),     // Top-right vertex
  vec2( -1.0, -1.0 ),   // Bottom-left vertex
  vec2( 1.0, -1.0 )     // Bottom-right vertex
);


void main()
{
  gl_Position = projectionMatrix * viewMatrix * vec4( positions[ gl_VertexID ], 0.0, 1.0 );
}
