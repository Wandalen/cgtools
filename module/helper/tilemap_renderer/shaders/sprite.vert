#version 300 es

layout( location = 1 ) in vec2 a_translation;
layout( location = 2 ) in vec2 a_rotation_cos_sin;
layout( location = 3 ) in vec2 a_scale;

uniform vec2 u_aspect_scale;

out vec2 v_tex_coord;

void main()
{
  const vec2[] VERTICES = vec2[]
  (
    vec2(  1.0, -1.0 ),
    vec2( -1.0, -1.0 ),
    vec2(  1.0,  1.0 ),
    vec2( -1.0,  1.0 )
  );
  vec2 position = VERTICES[ gl_VertexID ];
  v_tex_coord = position * 0.5 + 0.5;
  mat2 rot = mat2
  (
    vec2( a_rotation_cos_sin.x, a_rotation_cos_sin.y ),
    vec2( -a_rotation_cos_sin.y, a_rotation_cos_sin.x )
  );
  vec2 pos = ( rot * ( a_scale * position ) + a_translation ) * u_aspect_scale;
  gl_Position = vec4( pos, 0.0, 1.0 );
}
