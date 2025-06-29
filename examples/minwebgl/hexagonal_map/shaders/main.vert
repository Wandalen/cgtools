#version 300 es

layout( location = 0 ) in vec2 a_position;
layout( location = 1 ) in vec2 a_translation;
layout( location = 2 ) in int a_player_id;

uniform vec2 u_scale;
uniform vec2 u_camera_pos;

flat out int v_player_id;

void main()
{
  v_player_id = a_player_id;
  vec2 pos = u_scale * ( a_position + a_translation + u_camera_pos );
  gl_Position = vec4( pos, 0.0, 1.0 );
}
