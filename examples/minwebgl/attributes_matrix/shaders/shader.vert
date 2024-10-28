#version 300 es

layout( std140 ) uniform TransformBlock
{
  mat2 u_trans;
};

layout( location=0 ) in vec4 a_position;
layout( location=1 ) in vec4 a_color;
layout( location=2 ) in mat3x2 a_trans;

// Bear in mind WebGL matrices are column-major, so natural flow in a flat buffer is actually transposed one for WebGL.
//
// 3 columns x 2 rows
// [ a b c ]
// [ d e f ]

out vec4 v_color;

void main()
{
  v_color = a_color;
  vec4 pos = a_position;
  pos.w = 1.0;
  pos.xy = ( a_trans * vec3( pos.xy, 1.0 ) );
  pos.xy = u_trans * pos.xy;
  gl_Position = pos;
}
