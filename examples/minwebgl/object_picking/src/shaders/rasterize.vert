#version 300 es

out vec2 v_texcoord;

void main()
{
  const vec2 VERTECES[] = vec2[]
  (
    vec2( -1.0, -1.0 ),
    vec2(  3.0, -1.0 ),
    vec2( -1.0,  3.0 )
  );
  const vec2 TEX_COORDS[] = vec2[]
  (
    vec2( 0.0, 0.0 ),
    vec2( 2.0, 0.0 ),
    vec2( 0.0, 2.0 )
  );

  int index = gl_VertexID;
  gl_Position = vec4( VERTECES[ index ], 0.0, 1.0 );
  v_texcoord = TEX_COORDS[ index ];
}