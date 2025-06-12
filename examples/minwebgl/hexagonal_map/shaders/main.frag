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

in vec2 v_tex_coord;
flat in float v_player_id;

uniform sampler2D u_sprite_sheet;

layout ( location = 0 ) out vec4 frag_color;

void main()
{
  vec4 color = texture( u_sprite_sheet, v_tex_coord );
  vec3 player_color = PALETTE[ int( v_player_id ) ];

  frag_color = vec4( mix( color.rgb , player_color, 1.0 ), 1.0 );
}
