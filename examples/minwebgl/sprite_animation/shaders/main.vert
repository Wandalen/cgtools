#version 300 es

vec4 data_array[ 4 ] = vec4[ 4 ]
(
  vec4(  0.25, -0.50,  1.0,  1.0 ),
  vec4( -0.25, -0.50,  0.0,  1.0 ),
  vec4(  0.25,  0.50,  1.0,  0.0 ),
  vec4( -0.25,  0.50,  0.0,  0.0 )
);

layout( location = 0 ) in float a_depth;

out vec2 v_tex_coord;
out float v_depth;

void main()
{
  vec4 data = data_array[ gl_VertexID ];

  v_depth = a_depth;
  v_tex_coord = data.zw;
  gl_Position = vec4( data.xy, 0.0, 1.0 );
}