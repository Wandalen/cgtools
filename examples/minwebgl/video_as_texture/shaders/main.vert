#version 300 es

// Define a static array of 4 vec4 vertices representing a rectangular quad
vec4 data_array[ 4 ] = vec4[ 4 ]
( //      x,    y,    u,    v
  vec4( -0.3,  0.5,  0.0,  0.0 ), // Top-left
  vec4(  0.3,  0.5,  1.0,  0.0 ), // Top-right
  vec4( -0.3, -0.5,  0.0,  1.0 ), // Bottom-left
  vec4(  0.3, -0.5,  1.0,  1.0 )  // Bottom-right
);

// Output texture coordinates to fragment shader
out vec2 v_tex_coord;

void main()
{
  // Select vertex data based on the current vertex ID
  vec4 data = data_array[ gl_VertexID ];

  // Pass texture coordinates to fragment shader
  v_tex_coord = data.zw;
  // Set vertex position
  gl_Position = vec4( data.xy, 0.0, 1.0 );
}
