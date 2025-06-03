#version 300 es

layout( location = 0 ) in vec2 a_position;
layout( location = 1 ) in vec2 a_translation;
layout( location = 2 ) in vec2 a_tex_coord;
layout( location = 3 ) in vec2 a_sheet_offset;
layout( location = 4 ) in vec2 a_sprite_size;

uniform vec2 u_scale;
uniform vec2 u_camera_pos;

out vec2 v_tex_coord;

void main()
{
  v_tex_coord = a_sheet_offset + a_tex_coord * a_sprite_size;
  vec2 pos = u_scale * ( a_position + a_translation + u_camera_pos );
  gl_Position = vec4( pos, 0.0, 1.0 );
}
