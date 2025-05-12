#version 300 es

out vec2 v_tex_coord;

void main()
{
  // const vec2 TEXCOORDS[] = vec2[]
  // (
  //   vec2( 0.0, 0.0 ),
  //   vec2( 2.0, 0.0 ),
  //   vec2( 0.0, 2.0 )
  // );
  // v_texcoord = TEXCOORDS[ gl_VertexID ];

  const vec2 VERTICES[] = vec2[]
  (
    vec2( -1.0, -1.0 ),
    vec2(  3.0, -1.0 ),
    vec2( -1.0,  3.0 )
  );
  v_tex_coord = VERTICES[ gl_VertexID ] * 0.5 + 0.5;
  gl_Position = vec4( VERTICES[ gl_VertexID ], 0.0, 1.0 );
}
