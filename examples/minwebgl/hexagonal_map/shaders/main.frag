#version 300 es
precision mediump float;

in highp vec2 v_tex_coord;

uniform sampler2D u_sprite_sheet;

layout ( location = 0 ) out vec4 frag_color;

void main()
{
  vec4 color = texture( u_sprite_sheet, v_tex_coord );
  frag_color = vec4( color.rgb, 1.0 );
}
