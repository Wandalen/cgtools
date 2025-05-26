#version 300 es

// Output texture coordinate to the fragment shader
out vec2 v_tex_coord;

// Hardcoded vertex positions for a screen-filling quad in clip space ( -1.0 to 1.0 ).
// The vertex ID ( gl_VertexID ) is used to select the correct vertex from this array.
const vec2 TEXCOORDS[] = vec2[]
(
  vec2( -1.0, -1.0 ), // Bottom-left
  vec2( 1.0, -1.0 ),  // Bottom-right
  vec2( -1.0,  1.0 ),  // Top-left

  vec2( -1.0,  1.0 ),  // Top-left ( second triangle )
  vec2( 1.0, -1.0 ),  // Bottom-right ( second triangle )
  vec2( 1.0,  1.0 )   // Top-right
);

void main()
{
  // Convert the clip space position ( -1.0 to 1.0 ) to texture coordinates ( 0.0 to 1.0 ).
  // This maps the bottom-left of the quad to ( 0.0, 0.0 ) and the top-right to ( 1.0, 1.0 ).
  v_tex_coord = TEXCOORDS[ gl_VertexID ] * 0.5 + 0.5;
  // Set the final vertex position in clip space.
  gl_Position = vec4( TEXCOORDS[ gl_VertexID ], 0.0, 1.0 );
}