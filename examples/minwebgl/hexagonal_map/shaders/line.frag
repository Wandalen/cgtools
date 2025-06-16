#version 300 es
precision mediump float;

in vec2 v_tex_coord;
flat in float v_player_id;

layout ( location = 0 ) out vec4 frag_color;

void main()
{
  frag_color = vec4( vec3( 0.0 ), 1.0 );
}
