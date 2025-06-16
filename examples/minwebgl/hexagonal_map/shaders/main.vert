#version 300 es

layout( location = 0 ) in vec2 a_position;
layout( location = 1 ) in vec2 a_translation;
layout( location = 2 ) in vec2 a_tex_coord;
layout( location = 3 ) in vec2 a_sprite_offset;
layout( location = 4 ) in vec2 a_sprite_size;
layout( location = 5 ) in int a_player_id;

uniform vec2 u_scale;
uniform vec2 u_camera_pos;

out vec2 v_tex_coord;
flat out int v_player_id;

void main()
{
  v_tex_coord = a_sprite_offset + a_tex_coord * a_sprite_size;
  v_player_id = a_player_id;
  vec2 pos = u_scale * ( a_position + a_translation + u_camera_pos );
  gl_Position = vec4( pos, 0.0, 1.0 );
}
