#version 300 es

layout( location = 0 ) in vec2 a_translation;
layout( location = 1 ) in float a_aspect;
layout( location = 2 ) in float a_scale;

out vec2 v_tex_coord;

void main()
{
  const vec2[] VERTICES = vec2[]
  (
    vec2(  1.0,  1.0 ),
    vec2( -1.0,  1.0 ),
    vec2(  1.0, -1.0 ),
    vec2( -1.0, -1.0 )
  );
  v_tex_coord = VERTICES[ gl_VertexID ] * 0.5 + 0.5 * vec2( a_aspect, 1.0 );
  gl_Position = vec4( VERTICES[ gl_VertexID ] * a_scale + a_translation, 0.0, 1.0 );
}
