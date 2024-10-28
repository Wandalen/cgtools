#version 300 es

uniform mat2 u_trans;

// Define a constant array of 3 points for the triangle
const vec2 points[ 3 ] = vec2[]
(
  vec2( 0.0, 0.5 ),   // Top vertex
  vec2( -0.5, -0.5 ), // Bottom-left vertex
  vec2( 0.5, -0.5 )  // Bottom-right vertex
);

void main()
{
  // Select the vertex position based on the vertex ID
  vec2 position = points[ gl_VertexID ];

  // Apply the transformation matrix and set the position
  gl_Position = vec4( u_trans * position, 0.0, 1.0 );
}
