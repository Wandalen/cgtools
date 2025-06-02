#version 300 es

out vec2 v_tex_coord;

// Draws big triangle to rasterize whole screen
void main()
{
  const vec2 VERTICES[] = vec2[]
  (
    vec2( -1.0, -1.0 ),
    vec2(  3.0, -1.0 ),
    vec2( -1.0,  3.0 )
  );
  v_tex_coord = VERTICES[ gl_VertexID ] * 0.5 + 0.5;
  gl_Position = vec4( VERTICES[ gl_VertexID ], 0.0, 1.0 );
}
