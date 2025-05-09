#version 300 es

out vec3 v_normal;

// This shader draws a large triangle to rasterize whole screen
void main()
{
  const vec2 VERTICES[] = vec2[]
  (
    vec2( -1.0, -1.0 ),
    vec2(  3.0, -1.0 ),
    vec2( -1.0,  3.0 )
  );
  // const vec2 TEXCOORDS[] = vec2[]
  // (
  //   vec2( 0.0, 0.0 ),
  //   vec2( 2.0, 0.0 ),
  //   vec2( 0.0, 2.0 )
  // );

  v_normal = vec3( 0.0, 0.0, 1.0 );
  gl_Position = vec4( VERTICES[ gl_VertexID ], 0.5, 1.0 );
}
