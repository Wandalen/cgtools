#version 300 es

vec4 data_array[ 4 ] = vec4[ 4 ]
(
  vec4( -0.3,  0.5,  0.0,  0.0 ),
  vec4(  0.3,  0.5,  1.0,  0.0 ),
  vec4( -0.3, -0.5,  0.0,  1.0 ),
  vec4(  0.3, -0.5,  1.0,  1.0 )
);

out vec2 v_tex_coord;

void main()
{
  vec4 data = data_array[ gl_VertexID ];

  v_tex_coord = data.zw;
  gl_Position = vec4( data.xy, 0.0, 1.0 );
}
