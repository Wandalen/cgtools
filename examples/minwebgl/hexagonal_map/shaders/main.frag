#version 300 es
#define PLAYER_COUNT 1
precision mediump float;

uniform vec3[ PLAYER_COUNT ] u_player_colors;

flat in int v_player_id;

layout ( location = 0 ) out vec4 frag_color;

void main()
{
  vec3 player_color = u_player_colors[ v_player_id ];
  frag_color = vec4( player_color, 1.0 );
}
