#version 300 es
precision mediump float;

const vec3[] PALETTE = vec3[]
(
  vec3( 0.7, 0.5, 0.5 ),
  vec3( 0.2, 0.7, 0.5 ),
  vec3( 0.1, 0.7, 0.9 ),
  vec3( 0.8, 0.8, 0.4 ),
  vec3( 0.8, 0.4, 0.8 ),
  vec3( 0.2, 0.8, 0.8 )
);

flat in int v_player_id;

layout ( location = 0 ) out vec4 frag_color;

void main()
{
  vec3 player_color = PALETTE[ v_player_id ];
  frag_color = vec4( player_color, 1.0 );
}
