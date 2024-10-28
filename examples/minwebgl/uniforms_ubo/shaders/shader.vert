#version 300 es

layout( std140 ) uniform TransformBlock
{
  mat2 u_trans;
};

const vec2 points[ 3 ] = vec2[]
(
  vec2( 0.0, 0.5 ),   // Top vertex
  vec2( -0.5, -0.5 ), // Bottom-left vertex
  vec2( 0.5, -0.5 )   // Bottom-right vertex
);

void main()
{
  vec2 position = points[ gl_VertexID ];
  gl_Position = vec4( u_trans * position, 0.0, 1.0 );
}
