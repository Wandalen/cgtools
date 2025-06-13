#version 300 es

layout( location = 0 ) in vec2 a_translation;
layout( location = 1 ) in vec2 a_halfsize;
layout( location = 2 ) in float a_scale;

uniform vec2 u_scale;
uniform vec2 u_camera_pos;

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
  position = u_scale * ( position * a_halfsize * a_scale + a_translation + u_camera_pos );
  gl_Position = vec4( position, 0.0, 1.0 );
}
