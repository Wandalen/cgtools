#version 300 es

precision mediump float;

uniform mediump sampler2DArray u_sampler;

in vec2 v_tex_coord;
in float v_depth;

out vec4 frag_color;

void main()
{
  frag_color = texture( u_sampler, vec3( v_tex_coord, v_depth ) );
  //frag_color = vec4( 1.0 );
}